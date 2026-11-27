use crate::arca::{self, format_cuit, normalize_cuit_key, InvoiceData};
use image::{DynamicImage, GrayImage, RgbImage};
use lopdf::{Dictionary, Object, Stream};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ExtractionResult {
    pub data: InvoiceData,
    pub used_qr: bool,
    pub used_ocr: bool,
}

#[allow(dead_code)]
pub fn extract_invoice_data(path: &Path) -> ExtractionResult {
    extract_invoice_data_excluding(path, None)
}

pub fn extract_invoice_data_excluding(
    path: &Path,
    exclude_cuit: Option<&str>,
) -> ExtractionResult {
    let path_buf = path.to_path_buf();
    let exclude_owned = exclude_cuit.map(str::to_string);
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        extract_invoice_data_inner(&path_buf, exclude_owned.as_deref())
    })) {
        Ok(result) => result,
        Err(_) => {
            log::error!(
                "Panic controlado durante extracción de {}",
                path_buf.display()
            );
            ExtractionResult {
                data: InvoiceData::default(),
                used_qr: false,
                used_ocr: false,
            }
        }
    }
}

fn extract_invoice_data_inner(path: &Path, exclude_cuit: Option<&str>) -> ExtractionResult {
    let mut data = InvoiceData::default();
    let mut used_qr = false;
    let mut used_ocr = false;
    let exclude_key = exclude_cuit.map(normalize_cuit_key).filter(|s| s.len() == 11);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let pdf_doc = if ext == "pdf" {
        open_pdf(path)
    } else {
        None
    };
    if pdf_doc.is_some() {
        log::info!(
            "PDF abierto (posible decrypt) {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("pdf")
        );
    }

    // 1) Bytes crudos: QR ARCA + CUIT/texto embebido (aunque lopdf falle al parsear)
    if let Ok(bytes) = std::fs::read(path) {
        if let Some(qr_data) = find_arca_in_bytes(&bytes) {
            used_qr = true;
            arca::merge_invoice_data(&mut data, qr_data);
        }
        if needs_ocr(&data) {
            let lossy = String::from_utf8_lossy(&bytes);
            let from_raw = parse_text_fields_excluding(&lossy, exclude_key.as_deref());
            if from_raw.cuit.is_some()
                || from_raw.number.is_some()
                || from_raw.amount.is_some()
                || from_raw.issue_date.is_some()
            {
                log::info!(
                    "Datos hallados en bytes crudos de {}",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("pdf")
                );
                arca::merge_invoice_data(&mut data, from_raw);
                if data.provider_name.is_none() {
                    data.provider_name = guess_provider_name(&lossy, data.cuit.as_deref());
                }
            }
        }
    }

    // 2) Texto nativo del PDF (facturas digitales; muchas vienen encriptadas con pass vacío)
    if let Some(ref doc) = pdf_doc {
        if let Some(pdf_text) = extract_pdf_text_from_doc(doc) {
            log::info!(
                "Texto PDF extraído ({} chars) de {}",
                pdf_text.len(),
                path.display()
            );
            let from_text = parse_text_fields_excluding(&pdf_text, exclude_key.as_deref());
            let source = from_text.source.clone();
            arca::merge_invoice_data(&mut data, from_text);
            if data.source.is_none() {
                data.source = source.or_else(|| Some("pdf-text".into()));
            }
            if data.provider_name.is_none() {
                data.provider_name = guess_provider_name(&pdf_text, data.cuit.as_deref());
            }
        }
    }

    // 2b) Pistas desde el nombre de archivo (importe / nro)
    arca::merge_invoice_data(&mut data, guess_from_filename(path));

    // 3) Imágenes embebidas / archivo imagen: QR visual (solo candidatas chicas)
    let images = load_candidate_images(path, &ext, pdf_doc.as_ref());
    log::info!(
        "Imágenes candidatas para QR/OCR en {}: {}",
        path.display(),
        images.len()
    );
    for img in &images {
        if used_qr {
            break;
        }
        if !is_qr_candidate(img) {
            continue;
        }
        if let Some(qr_data) = decode_qr_from_image_safe(img) {
            used_qr = true;
            arca::merge_invoice_data(&mut data, qr_data);
        }
    }

    // 4) OCR con Tesseract (escaneos / si faltan CUIT u otros campos)
    if needs_ocr(&data) {
        if let Some(ocr_text) = run_ocr_on_file(path, &ext) {
            used_ocr = true;
            apply_ocr_text_excluding(&mut data, &ocr_text, exclude_key.as_deref());
        } else {
            for (idx, img) in images.into_iter().take(4).enumerate() {
                // Evitar OCR de láminas enormes (pueden tumbar el proceso)
                if img.width() > 1800 || img.height() > 1800 {
                    continue;
                }
                if let Some(ocr_text) = ocr_dynamic_image(&img, idx) {
                    used_ocr = true;
                    apply_ocr_text_excluding(&mut data, &ocr_text, exclude_key.as_deref());
                    if !needs_ocr(&data) {
                        break;
                    }
                }
            }
        }
    }

    if let Some(ref key) = exclude_key {
        if data
            .cuit
            .as_ref()
            .map(|c| normalize_cuit_key(c) == *key)
            .unwrap_or(false)
        {
            log::warn!(
                "CUIT extraído coincide con la entidad; se descarta para proveedor"
            );
            data.cuit = None;
        }
    }

    let file_label = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archivo");
    if data.cuit.is_some() {
        log::info!(
            "Extracción {}: nro={:?} qr={} ocr={}",
            file_label,
            data.number,
            used_qr,
            used_ocr
        );
    } else {
        log::warn!("Sin CUIT en {}: qr={} ocr={}", file_label, used_qr, used_ocr);
    }

    if data.doc_type.is_none() {
        data.doc_type = Some("factura".into());
    }
    if data.currency.is_none() {
        data.currency = Some("ARS".into());
    }

    ExtractionResult {
        data,
        used_qr,
        used_ocr,
    }
}

fn needs_ocr(data: &InvoiceData) -> bool {
    data.cuit.is_none() || data.number.is_none() || data.amount.is_none() || data.issue_date.is_none()
}

fn apply_ocr_text_excluding(data: &mut InvoiceData, ocr_text: &str, exclude_cuit: Option<&str>) {
    let from_ocr = parse_text_fields_excluding(ocr_text, exclude_cuit);
    arca::merge_invoice_data(data, from_ocr);
    if data.provider_name.is_none() {
        data.provider_name = guess_provider_name(ocr_text, data.cuit.as_deref());
    }
}

fn open_pdf(path: &Path) -> Option<lopdf::Document> {
    let mut doc = lopdf::Document::load(path).ok()?;
    if doc.is_encrypted() {
        match doc.decrypt("") {
            Ok(()) => log::info!(
                "PDF desencriptado con password vacío: {}",
                path.file_name().and_then(|n| n.to_str()).unwrap_or("pdf")
            ),
            Err(err) => {
                log::warn!(
                    "PDF encriptado y no se pudo abrir con pass vacío ({}): {}",
                    path.display(),
                    err
                );
                return None;
            }
        }
    }
    Some(doc)
}

fn extract_pdf_text_from_doc(doc: &lopdf::Document) -> Option<String> {
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    if pages.is_empty() {
        return None;
    }
    // Limitar a primeras páginas para evitar panics/loops en PDFs raros
    let page_slice: Vec<u32> = pages.into_iter().take(3).collect();
    let text = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        doc.extract_text(&page_slice)
    })) {
        Ok(Ok(t)) => t,
        Ok(Err(err)) => {
            log::warn!("extract_text falló: {err}");
            String::new()
        }
        Err(_) => {
            log::warn!("Panic en extract_text de PDF");
            String::new()
        }
    };
    let mut trimmed = text.trim().to_string();
    // lopdf extract_text a veces omite bloques (o falla del todo). Completar con literales Tj.
    if let Some(fallback) = extract_pdf_tj_strings(doc) {
        let low = trimmed.to_ascii_lowercase();
        let need_fallback = trimmed.len() < 80
            || !low.contains("vencimiento")
            || !low.contains("total")
            || fallback.len() > trimmed.len().saturating_mul(2);
        if need_fallback {
            log::info!(
                "Texto PDF + Tj fallback (extract={}, tj={})",
                trimmed.len(),
                fallback.len()
            );
            if trimmed.is_empty() {
                trimmed = fallback;
            } else {
                trimmed = format!("{trimmed}\n{fallback}");
            }
        }
    }
    if trimmed.len() < 8 {
        return None;
    }
    Some(trimmed)
}

fn extract_pdf_tj_strings(doc: &lopdf::Document) -> Option<String> {
    let mut chunks: Vec<String> = Vec::new();
    for object in doc.objects.values() {
        let Object::Stream(stream) = object else {
            continue;
        };
        // Evitar streams de imagen
        if let Ok(Object::Name(filter)) = stream.dict.get(b"Filter") {
            let f = String::from_utf8_lossy(filter).to_ascii_lowercase();
            if f.contains("dct") || f.contains("jpx") || f.contains("ccitt") || f.contains("jbig") {
                continue;
            }
        }
        if stream.dict.get(b"Width").is_ok() && stream.dict.get(b"Height").is_ok() {
            continue;
        }
        let data = match stream.decompressed_content() {
            Ok(d) => d,
            Err(_) => stream.content.clone(),
        };
        if data.len() < 40 || data.len() > 4_000_000 {
            continue;
        }
        let lossy = String::from_utf8_lossy(&data);
        if !lossy.contains("Tj") && !lossy.contains("TJ") {
            continue;
        }
        collect_pdf_literal_strings(&lossy, &mut chunks);
    }
    if chunks.is_empty() {
        return None;
    }
    let joined = chunks.join(" ");
    let trimmed = joined.trim().to_string();
    if trimmed.len() < 20 {
        None
    } else {
        Some(trimmed)
    }
}

fn collect_pdf_literal_strings(content: &str, out: &mut Vec<String>) {
    // (texto) Tj  — incluye escapes PDF \( \) \\
    let Ok(re) = Regex::new(r"\((?:\\.|[^\\)])*\)\s*Tj") else {
        return;
    };
    for m in re.find_iter(content) {
        let raw = m.as_str();
        let inner = raw
            .trim_end_matches("Tj")
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')');
        let decoded = decode_pdf_literal(inner);
        if decoded.chars().any(|c| c.is_alphanumeric()) {
            out.push(decoded);
        }
    }
}

fn decode_pdf_literal(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('(') => out.push('('),
                Some(')') => out.push(')'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => break,
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn guess_from_filename(path: &Path) -> InvoiceData {
    let mut data = InvoiceData {
        source: Some("filename".into()),
        ..Default::default()
    };
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return data;
    }

    if let Ok(re) = Regex::new(r"(?i)\bFC\s*(\d{1,8})\b") {
        if let Some(caps) = re.captures(&name) {
            data.number = Some(caps[1].to_string());
            data.doc_type = Some("Factura C".into());
        }
    }
    if data.number.is_none() {
        if let Ok(re) = Regex::new(r"(?i)\b(?:FCC|FCA|FCB|NCC|NCA|NCB)\s*[-_]?(\d{4,8})\b") {
            if let Some(caps) = re.captures(&name) {
                data.number = Some(caps[1].to_string());
            }
        }
    }

    if let Ok(re) = Regex::new(r"\$\s*([0-9]{1,3}(?:\.[0-9]{3})*(?:,[0-9]{2})?|[0-9]+(?:[.,][0-9]{2})?)")
    {
        if let Some(caps) = re.captures(&name) {
            data.amount = parse_amount(&caps[1]);
        }
    }

    data
}

fn find_arca_in_bytes(bytes: &[u8]) -> Option<InvoiceData> {
    let text = String::from_utf8_lossy(bytes);
    // URLs pueden venir fragmentadas; buscamos patrones p= base64 largos
    let re = Regex::new(
        r#"(?i)(?:afip\.gob\.ar|arca\.gob\.ar)[^"'\s<>]{0,80}\?p=([A-Za-z0-9+/=]{40,})"#,
    )
    .ok()?;
    if let Some(caps) = re.captures(&text) {
        let fake_url = format!("https://www.afip.gob.ar/fe/qr/?p={}", &caps[1]);
        if let Some(data) = arca::parse_arca_qr_content(&fake_url) {
            return Some(data);
        }
    }

    let re_p = Regex::new(r"\?p=([A-Za-z0-9+/=]{60,})").ok()?;
    for caps in re_p.captures_iter(&text) {
        let fake_url = format!("https://www.afip.gob.ar/fe/qr/?p={}", &caps[1]);
        if let Some(data) = arca::parse_arca_qr_content(&fake_url) {
            return Some(data);
        }
    }
    None
}

fn is_qr_candidate(img: &DynamicImage) -> bool {
    let w = img.width();
    let h = img.height();
    if w < 80 || h < 80 || w > 900 || h > 900 {
        return false;
    }
    // QR AFIP suele ser cuadrado o casi cuadrado
    let ratio = w as f32 / h as f32;
    (0.75..=1.35).contains(&ratio)
}

fn decode_qr_from_image_safe(img: &DynamicImage) -> Option<InvoiceData> {
    let owned = img.clone();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode_qr_from_image(&owned)))
    {
        Ok(v) => v,
        Err(_) => {
            log::warn!("Panic en rqrr al leer QR de imagen {}x{}", img.width(), img.height());
            None
        }
    }
}

fn decode_qr_from_image(img: &DynamicImage) -> Option<InvoiceData> {
    let gray = img.to_luma8();
    let mut variants = vec![DynamicImage::ImageLuma8(gray.clone())];

    // Solo escalar si el QR es chico (evita OOM)
    if img.width() <= 420 && img.height() <= 420 {
        variants.push(DynamicImage::ImageLuma8(image::imageops::resize(
            &gray,
            img.width().saturating_mul(2).max(1),
            img.height().saturating_mul(2).max(1),
            image::imageops::FilterType::Nearest,
        )));
    }

    let mut inv = gray;
    for p in inv.pixels_mut() {
        p[0] = 255 - p[0];
    }
    variants.push(DynamicImage::ImageLuma8(inv));

    for variant in variants {
        let luma = variant.to_luma8();
        let mut prepared = rqrr::PreparedImage::prepare(luma);
        for grid in prepared.detect_grids() {
            if let Ok((_, content)) = grid.decode() {
                if let Some(data) = arca::parse_arca_qr_content(&content) {
                    log::info!("QR ARCA decodificado desde imagen");
                    return Some(data);
                }
                let fake = format!("https://www.afip.gob.ar/fe/qr/?p={content}");
                if let Some(data) = arca::parse_arca_qr_content(&fake) {
                    log::info!("QR ARCA (payload crudo) decodificado desde imagen");
                    return Some(data);
                }
            }
        }
    }
    None
}

fn load_candidate_images(
    path: &Path,
    ext: &str,
    pdf_doc: Option<&lopdf::Document>,
) -> Vec<DynamicImage> {
    let mut images = Vec::new();
    if matches!(ext, "png" | "jpg" | "jpeg" | "webp" | "gif") {
        if let Ok(img) = image::open(path) {
            images.push(img);
        }
        return images;
    }

    if ext != "pdf" {
        return images;
    }

    let owned;
    let doc = match pdf_doc {
        Some(d) => d,
        None => {
            owned = open_pdf(path);
            match owned.as_ref() {
                Some(d) => d,
                None => return images,
            }
        }
    };

    for (_id, object) in doc.objects.iter() {
        let Object::Stream(stream) = object else {
            continue;
        };
        if let Some(img) = pdf_stream_to_image(stream) {
            // Priorizar cuadrados típicos de QR (~200-500px)
            let w = img.width();
            let h = img.height();
            if w == h && (180..=520).contains(&w) {
                images.insert(0, img);
            } else {
                images.push(img);
            }
        }
    }

    // Limitar para no OCR/QR-escanear decenas de imágenes enormes
    if images.len() > 12 {
        images.truncate(12);
    }
    images
}

fn pdf_stream_to_image(stream: &Stream) -> Option<DynamicImage> {
    let dict = &stream.dict;
    let subtype = dict.get(b"Subtype").ok().and_then(|o| o.as_name().ok())?;
    if subtype != b"Image" {
        return None;
    }

    let width = dict_u32(dict, b"Width")?;
    let height = dict_u32(dict, b"Height")?;
    if width == 0 || height == 0 || width > 4000 || height > 4000 {
        return None;
    }

    let filter_name = dict
        .get(b"Filter")
        .ok()
        .and_then(|f| match f {
            Object::Name(n) => Some(n.as_slice()),
            Object::Array(arr) => arr.first().and_then(|o| o.as_name().ok()),
            _ => None,
        })
        .map(|n| n.to_vec());

    let decoded = stream
        .decompressed_content()
        .unwrap_or_else(|_| stream.content.clone());

    if filter_name
        .as_deref()
        .is_some_and(|n| n == b"DCTDecode" || n == b"JPXDecode")
        || decoded.starts_with(&[0xFF, 0xD8])
    {
        if let Ok(img) = image::load_from_memory(&decoded) {
            return Some(img);
        }
    }

    let bpc = dict
        .get(b"BitsPerComponent")
        .ok()
        .and_then(|o| o.as_i64().ok())
        .unwrap_or(8) as u32;
    if bpc != 8 {
        return None;
    }

    let cs = color_space_name(dict);
    match cs.as_deref() {
        Some("DeviceGray") | Some("Gray") => {
            let need = (width as usize).saturating_mul(height as usize);
            if decoded.len() < need {
                return None;
            }
            GrayImage::from_raw(width, height, decoded[..need].to_vec()).map(DynamicImage::ImageLuma8)
        }
        Some("DeviceRGB") | Some("RGB") => {
            let need = (width as usize)
                .saturating_mul(height as usize)
                .saturating_mul(3);
            if decoded.len() < need {
                return None;
            }
            RgbImage::from_raw(width, height, decoded[..need].to_vec()).map(DynamicImage::ImageRgb8)
        }
        _ => None,
    }
}

fn dict_u32(dict: &Dictionary, key: &[u8]) -> Option<u32> {
    dict.get(key)
        .ok()
        .and_then(|o| o.as_i64().ok())
        .and_then(|v| u32::try_from(v).ok())
}

fn color_space_name(dict: &Dictionary) -> Option<String> {
    let obj = dict.get(b"ColorSpace").ok()?;
    match obj {
        Object::Name(n) => Some(String::from_utf8_lossy(n).into_owned()),
        Object::Array(arr) => {
            if let Some(Object::Name(n)) = arr.first() {
                let name = String::from_utf8_lossy(n);
                if name == "Indexed" {
                    // Indexed no lo convertimos acá; el QR suele estar en DeviceRGB/Gray.
                    return Some("Indexed".into());
                }
                return Some(name.into_owned());
            }
            None
        }
        _ => None,
    }
}

fn run_ocr_on_file(path: &Path, ext: &str) -> Option<String> {
    if matches!(ext, "png" | "jpg" | "jpeg" | "tif" | "tiff" | "webp" | "bmp") {
        return run_tesseract(path);
    }
    None
}

fn ocr_dynamic_image(img: &DynamicImage, idx: usize) -> Option<String> {
    let dir = std::env::temp_dir().join("folio_ocr");
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join(format!("page_{idx}.png"));
    img.save(&file).ok()?;
    let text = run_tesseract(&file);
    let _ = std::fs::remove_file(&file);
    text
}

fn run_tesseract(image_path: &Path) -> Option<String> {
    let tess = find_tesseract()?;
    log::info!(
        "OCR Tesseract: {}",
        image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("imagen")
    );
    if let Some(text) = run_tesseract_lang(&tess, image_path, "spa+eng") {
        return Some(text);
    }
    run_tesseract_lang(&tess, image_path, "eng")
}

fn run_tesseract_lang(tess: &Path, image_path: &Path, lang: &str) -> Option<String> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let mut child = Command::new(tess)
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .arg("--psm")
        .arg("6")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    });

    let started = Instant::now();
    let timeout = Duration::from_secs(45);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let text = reader.join().ok()?;
                if !status.success() {
                    log::warn!(
                        "Tesseract ({lang}) falló para {}",
                        image_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("imagen")
                    );
                    return None;
                }
                return Some(text);
            }
            Ok(None) => {
                if started.elapsed() > timeout {
                    log::warn!(
                        "Tesseract ({lang}) timeout 45s en {}",
                        image_path.display()
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = reader.join();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err) => {
                log::warn!("Tesseract error: {err}");
                let _ = child.kill();
                let _ = reader.join();
                return None;
            }
        }
    }
}

fn find_tesseract() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(r"C:\Program Files\Tesseract-OCR\tesseract.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"),
        PathBuf::from("tesseract"),
    ];
    for c in candidates {
        if c.file_name().is_some() && (c.exists() || c == PathBuf::from("tesseract")) {
            if c == PathBuf::from("tesseract") {
                return Some(c);
            }
            if c.exists() {
                return Some(c);
            }
        }
    }
    None
}

/// Parseo de texto libre (PDF.js / OCR / visor) → campos de factura.
pub fn parse_invoice_text(text: &str, exclude_cuit: Option<&str>) -> InvoiceData {
    parse_text_fields_excluding(text, exclude_cuit)
}

fn parse_text_fields_excluding(text: &str, exclude_cuit: Option<&str>) -> InvoiceData {
    let mut data = InvoiceData {
        source: Some("text".into()),
        ..Default::default()
    };

    // PDFs suelen partir fechas/montos entre líneas (ej. 17/11/202\n6)
    let compact = normalize_pdf_text(text);

    if let Some(cuit) = find_cuit(&compact, exclude_cuit) {
        data.cuit = Some(cuit);
    }

    // Nro 0001-00001234 / Comp. Nro:00001 00000047
    if let Ok(re) = Regex::new(
        r"(?i)(?:n[ºo°\.]?\s*|comp(?:robante)?\.?\s*n(?:ro|um(?:ero)?)?\.?\s*:?\s*)(\d{4,5})\s*[-/\s]\s*(\d{6,8})",
    ) {
        if let Some(caps) = re.captures(&compact) {
            data.point_of_sale = caps[1].parse().ok();
            data.number = Some(format!("{}-{}", &caps[1], &caps[2]));
        }
    }
    if data.number.is_none() {
        if let Ok(re) = Regex::new(r"\b(\d{4,5})\s*[-/]\s*(\d{8})\b") {
            if let Some(caps) = re.captures(&compact) {
                data.point_of_sale = caps[1].parse().ok();
                data.number = Some(format!("{}-{}", &caps[1], &caps[2]));
            }
        }
    }
    if data.point_of_sale.is_none() {
        if let Ok(re) = Regex::new(r"(?i)punto\s*de\s*venta\s*:?\s*(\d{1,5})") {
            if let Some(caps) = re.captures(&compact) {
                data.point_of_sale = caps[1].parse().ok();
            }
        }
    }

    if let Ok(re) = Regex::new(r"(?i)\bC\.?A\.?E\.?\s*N?[º°o.]?\s*:?\s*(\d{10,14})\b") {
        if let Some(caps) = re.captures(&compact) {
            data.auth_type = Some("CAE".into());
            data.auth_code = Some(caps[1].to_string());
        }
    }

    data.issue_date = find_issue_date(&compact);
    if let Some(ref d) = data.issue_date {
        if d.len() >= 7 {
            data.period = Some(d[..7].to_string());
        }
    }

    data.due_date = find_due_date(&compact);
    data.amount = find_amount(&compact);

    // Servicios (AySA/EDESUR): cuenta / LSP como número si no hay comprobante AFIP
    if data.number.is_none() {
        if let Ok(re) = Regex::new(r"(?i)\bLSP\s*([0-9A-Z\-]+)") {
            if let Some(caps) = re.captures(&compact) {
                data.number = Some(caps[1].to_string());
                data.doc_type = Some("liquidación servicio".into());
            }
        }
    }
    if data.number.is_none() {
        if let Ok(re) = Regex::new(r"(?i)cuenta\s*de\s*servicios\s*:?\s*(\d{5,10})") {
            if let Some(caps) = re.captures(&compact) {
                data.number = Some(caps[1].to_string());
                data.doc_type = Some("liquidación servicio".into());
            }
        }
    }

    data
}

fn normalize_pdf_text(text: &str) -> String {
    let mut s = text.replace('\r', "\n");
    // UTF-16LE/BE leído como latin1: "F\0A\0C\0T" → "FACT"
    if s.chars().filter(|c| *c == '\0').count() > 20 {
        s = s.chars().filter(|c| *c != '\0').collect();
    }
    // lopdf a veces emite "000F000A000C000T" por glifos/CID
    if let Ok(re) = Regex::new(r"(?:000)([A-Za-z0-9ÁÉÍÓÚáéíóúÑñ/:\-.,$% ])") {
        let hits = re.find_iter(&s).count();
        if hits >= 12 {
            s = re.replace_all(&s, "$1").into_owned();
        }
    }
    // Reunir dígitos partidos por salto de línea
    if let Ok(re) = Regex::new(r"(\d)\n+(\d)") {
        loop {
            let next = re.replace_all(&s, "$1$2").into_owned();
            if next == s {
                break;
            }
            s = next;
        }
    }
    // "$" solo en una línea y el monto en la siguiente
    if let Ok(re) = Regex::new(r"\$\s*\n+\s*") {
        s = re.replace_all(&s, "$ ").into_owned();
    }
    // Visores/OCR a veces pegan sin espacios: Totalapagar, Vencimiento13/07/2026
    s = expand_compact_invoice_labels(&s);
    // Colapsar whitespace (incluye newlines) para etiquetas/valor
    if let Ok(re) = Regex::new(r"\s+") {
        s = re.replace_all(&s, " ").into_owned();
    }
    s
}

fn expand_compact_invoice_labels(text: &str) -> String {
    let replacements = [
        (r"(?i)totalapagarhastael", " total a pagar hasta el "),
        (r"(?i)totalapagarconrecargohastael", " total a pagar con recargo hasta el "),
        (r"(?i)totalapagar", " total a pagar "),
        (r"(?i)importetotal", " importe total "),
        (r"(?i)totalfactura", " total factura "),
        (r"(?i)saldoapagar", " saldo a pagar "),
        (r"(?i)fechadeemisi[oó]n", " fecha de emision "),
        (r"(?i)fechavto", " fecha vto "),
        (r"(?i)vencimiento", " vencimiento "),
        (r"(?i)cuentadeservicios", " cuenta de servicios "),
        (r"(?i)cuitn[ºo°]?", " CUIT "),
    ];
    let mut s = text.to_string();
    for (pat, rep) in replacements {
        if let Ok(re) = Regex::new(pat) {
            s = re.replace_all(&s, rep).into_owned();
        }
    }
    s
}

fn find_issue_date(text: &str) -> Option<String> {
    let patterns = [
        r"(?i)fecha\s*(?:de\s*)?emisi[oó]n\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4}|\d{4}-\d{2}-\d{2})",
        r"(?i)fecha\s*(?:de\s*)?emisi[oó]n\s*:?[^\d]{0,40}(\d{2}[/-]\d{2}[/-]\d{2,4})",
    ];
    for pat in patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(caps) = re.captures(text) {
                if let Some(d) = normalize_parsed_date(&caps[1]) {
                    return Some(d);
                }
            }
        }
    }
    None
}

fn find_due_date(text: &str) -> Option<String> {
    // Priorizar etiquetas de vencimiento de PAGO (no CAE)
    let preferred = [
        r"(?i)fecha\s*(?:de\s*)?vto\.?\s*para\s*el\s*pago\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)vto\.?\s*para\s*el\s*pago\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)total\s*a\s*pagar\s*hasta\s*(?:el\s*)?(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)al\s*vto\.?\s*l[ií]mite\s*de\s*pago[^0-9]{0,40}(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)fecha\s*vto\.?\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)(?:1er\.?\s*)?vencimiento\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4})",
        r"(?i)\bvto\.?\s*:?\s*(\d{2}[/-]\d{2}[/-]\d{2,4})",
    ];
    for pat in preferred {
        if let Ok(re) = Regex::new(pat) {
            for caps in re.captures_iter(text) {
                if is_cae_due_context(text, caps.get(0).map(|m| m.start()).unwrap_or(0)) {
                    continue;
                }
                if let Some(d) = normalize_parsed_date(&caps[1]) {
                    return Some(d);
                }
            }
        }
    }
    None
}

fn is_cae_due_context(text: &str, match_start: usize) -> bool {
    // Solo la vecindad inmediata (misma etiqueta CAE … VENCIMIENTO), no líneas previas
    let from = match_start.saturating_sub(28);
    let window = text.get(from..match_start).unwrap_or("");
    if window.contains('\n') {
        // Si hay salto de línea, mirar solo desde el último salto
        let line_start = window.rfind('\n').map(|i| from + i + 1).unwrap_or(from);
        let line_window = text.get(line_start..match_start).unwrap_or("");
        let lower = line_window.to_ascii_lowercase();
        return lower.contains("cae") || lower.contains("c.a.e");
    }
    let lower = window.to_ascii_lowercase();
    lower.contains("cae") || lower.contains("c.a.e")
}

fn find_amount(text: &str) -> Option<f64> {
    // Orden: totales de pago / factura primero; evitar subtotales e IVA
    let patterns = [
        r"(?i)total\s*a\s*pagar\s*hasta\s*(?:el\s*)?\d{2}[/-]\d{2}[/-]\d{2,4}\s*:?\s*\$?\s*([0-9]{1,3}(?:[.,\s][0-9]{3})*(?:[.,][0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
        r"(?i)total\s*a\s*pagar\s*:?\s*\$?\s*([0-9]{1,3}(?:[.,\s][0-9]{3})*(?:[.,][0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
        r"(?i)importe\s*total\s*:?\s*\$?\s*([0-9]{1,3}(?:[.,\s][0-9]{3})*(?:[.,][0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
        r"(?i)total\s*factura\s*:?\s*\$?\s*([0-9]{1,3}(?:[.,\s][0-9]{3})*(?:[.,][0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
        r"(?i)saldo\s*a\s*pagar\s*:?\s*\$?\s*([0-9]{1,3}(?:[.,\s][0-9]{3})*(?:[.,][0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
        r"(?i)(?:^|[^\w])total\s*:?\s*\$?\s*([0-9]{1,3}(?:[.\s][0-9]{3})*(?:,[0-9]{2})|[0-9]+(?:,[0-9]{2}))",
    ];

    let mut candidates: Vec<(i32, f64)> = Vec::new();
    for (prio, pat) in patterns.iter().enumerate() {
        if let Ok(re) = Regex::new(pat) {
            for caps in re.captures_iter(text) {
                let raw = &caps[1];
                let full = caps.get(0).map(|m| m.as_str()).unwrap_or("").to_ascii_lowercase();
                if full.contains("subtotal")
                    || full.contains("iva")
                    || full.contains("tributos")
                    || full.contains("exento")
                    || full.contains("neto")
                {
                    continue;
                }
                if let Some(v) = parse_amount(raw) {
                    // CLARO: "Total a Pagar: $0,00" no es el total real
                    if v == 0.0 && prio <= 1 {
                        continue;
                    }
                    candidates.push((prio as i32, v));
                }
            }
        }
    }

    // EDESUR / servicios: "Al vto. ... $ 667,834.46"
    if let Ok(re) = Regex::new(
        r"(?i)al\s*vto\.?[^\n$]{0,60}\$\s*([0-9]{1,3}(?:,[0-9]{3})+(?:\.[0-9]{2})|[0-9]+(?:[.,][0-9]{2}))",
    ) {
        if let Some(caps) = re.captures(text) {
            if let Some(v) = parse_amount(&caps[1]) {
                candidates.push((0, v));
            }
        }
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)));
    candidates.into_iter().next().map(|(_, v)| v)
}

fn normalize_parsed_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if let Ok(re) = Regex::new(r"^(\d{2})[/-](\d{2})[/-](\d{2,4})$") {
        if let Some(caps) = re.captures(raw) {
            let d = &caps[1];
            let m = &caps[2];
            let mut y = caps[3].to_string();
            if y.len() == 2 {
                let n: i32 = y.parse().ok()?;
                y = if n >= 70 {
                    format!("{}", 1900 + n)
                } else {
                    format!("{}", 2000 + n)
                };
            }
            let month: i32 = m.parse().ok()?;
            let day: i32 = d.parse().ok()?;
            if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
                return None;
            }
            return Some(format!("{y}-{m}-{d}"));
        }
    }
    if let Ok(re) = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$") {
        if let Some(caps) = re.captures(raw) {
            return Some(format!("{}-{}-{}", &caps[1], &caps[2], &caps[3]));
        }
    }
    None
}

fn find_cuit(text: &str, exclude_cuit: Option<&str>) -> Option<String> {
    let exclude = exclude_cuit.unwrap_or("");
    let keep = |digits: &str| exclude.is_empty() || digits != exclude;

    // Preferir CUIT etiquetado (emisor)
    let labeled = Regex::new(
        r"(?i)(?:C\.?\s*U\.?\s*I\.?\s*T\.?|CUIT)\s*(?:N[ºO°\.]?)?\s*:?\s*(\d{2})[-\.\s]?(\d{8})[-\.\s]?(\d)",
    )
    .ok()?;
    for caps in labeled.captures_iter(text) {
        let digits = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
        if digits.len() == 11 && keep(&digits) {
            return Some(format_cuit(&digits));
        }
    }

    let re = Regex::new(r"\b(\d{2})[-\.\s]?(\d{8})[-\.\s]?(\d)\b").ok()?;
    let mut first: Option<String> = None;
    let mut preferred_person: Option<String> = None;
    for caps in re.captures_iter(text) {
        let digits = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
        if digits.len() != 11 || !keep(&digits) {
            continue;
        }
        let formatted = format_cuit(&digits);
        // En facturas recibidas el emisor suele ser 20/23/24/27…
        if matches!(&caps[1], "20" | "23" | "24" | "27") && preferred_person.is_none() {
            preferred_person = Some(formatted.clone());
        }
        if first.is_none() {
            first = Some(formatted);
        }
    }
    preferred_person.or(first)
}

fn parse_amount(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();
    if cleaned.is_empty() {
        return None;
    }

    let last_comma = cleaned.rfind(',');
    let last_dot = cleaned.rfind('.');

    // Ambos separadores: el último es decimal
    if let (Some(c), Some(d)) = (last_comma, last_dot) {
        let normalized = if c > d {
            // 1.234.567,89 (AR)
            cleaned.replace('.', "").replace(',', ".")
        } else {
            // 667,834.46 (US / EDESUR)
            cleaned.replace(',', "")
        };
        return normalized.parse().ok();
    }

    if cleaned.contains(',') {
        let parts: Vec<&str> = cleaned.split(',').collect();
        if parts.len() == 2 && parts[1].len() <= 2 {
            return format!("{}.{}", parts[0].replace('.', ""), parts[1])
                .parse()
                .ok();
        }
        // 1,234,567 miles US sin decimales
        if parts.iter().skip(1).all(|p| p.len() == 3) {
            return cleaned.replace(',', "").parse().ok();
        }
        return cleaned.replace(',', ".").parse().ok();
    }

    if cleaned.contains('.') {
        let parts: Vec<&str> = cleaned.split('.').collect();
        // 1.234.567 o 2.970 / 99.360 → miles AR sin decimales
        if parts.len() > 1 && parts.iter().skip(1).all(|p| p.len() == 3) {
            return cleaned.replace('.', "").parse().ok();
        }
        // 1234.56 decimal
        if parts.len() == 2 && (1..=2).contains(&parts[1].len()) {
            return cleaned.parse().ok();
        }
    }

    cleaned.parse().ok()
}

fn guess_provider_name(text: &str, cuit: Option<&str>) -> Option<String> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.len() >= 4 && l.len() <= 80)
        .collect();

    for (idx, line) in lines.iter().enumerate() {
        let upper = line.to_ascii_uppercase();
        if upper.contains("CUIT") || upper.contains("C.U.I.T") {
            // línea anterior suele ser razón social
            if idx > 0 {
                let prev = lines[idx - 1];
                if looks_like_name(prev) {
                    return Some(prev.to_string());
                }
            }
        }
    }

    // Primera línea “nombre” razonable
    for line in lines.iter().take(12) {
        if looks_like_name(line) {
            return Some((*line).to_string());
        }
    }

    cuit.map(|c| format!("Proveedor {c}"))
}

fn looks_like_name(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    if upper.contains("FACTURA")
        || upper.contains("CODIGO")
        || upper.contains("PÁGINA")
        || upper.contains("PAGINA")
        || upper.contains("CUIT")
        || upper.starts_with("IVA")
        || line.chars().filter(|c| c.is_ascii_digit()).count() > line.len() / 2
    {
        return false;
    }
    line.chars().any(|c| c.is_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_amount_ar_and_us() {
        assert_eq!(parse_amount("2.970"), Some(2970.0));
        assert_eq!(parse_amount("99.360"), Some(99360.0));
        assert_eq!(parse_amount("160.000,00"), Some(160000.0));
        assert_eq!(parse_amount("193.756,49"), Some(193756.49));
        assert_eq!(parse_amount("667,834.46"), Some(667834.46));
        assert_eq!(parse_amount("99360,00"), Some(99360.0));
        assert_eq!(parse_amount("1.400.000,00"), Some(1400000.0));
    }

    #[test]
    fn due_date_prefers_payment_not_cae() {
        let text = "CAE N°: 86474244215569 VENCIMIENTO: 04/12/2026\nVENCIMIENTO: 01/12/2026 TOTAL 160.000,00";
        // El primer VENCIMIENTO suelto (pago) debe ganar sobre el del CAE
        assert_eq!(find_due_date(text).as_deref(), Some("2026-12-01"));
    }

    #[test]
    fn amount_total_a_pagar_edesur() {
        let text = "Total a Pagar Hasta el 17/11/2026   667,834.46\nDespués del vencimiento 718,098.34";
        assert_eq!(find_amount(&normalize_pdf_text(text)), Some(667834.46));
    }

    #[test]
    fn edesur_split_date_and_amount() {
        let text = "Subtotal Cargos del Mes\n667,834.46\nTotal a Pagar Hasta el 17/11/202\n6   \n667,834.46\nRecargo por mora 5%";
        let flat = normalize_pdf_text(text);
        assert_eq!(find_due_date(&flat).as_deref(), Some("2026-11-17"));
        assert_eq!(find_amount(&flat), Some(667834.46));
    }

    #[test]
    fn amount_skips_zero_total_a_pagar() {
        let text = "Total a Pagar: $0,00\nTotal Factura: $193.756,49\nVencimiento: 24/07/2026";
        assert_eq!(find_amount(text), Some(193756.49));
        assert_eq!(find_due_date(text).as_deref(), Some("2026-07-24"));
    }

    #[test]
    fn aysa_compact_and_spaced_text() {
        let compact = "Vencimiento 13/07/2026 Totalapagar $204.843,80 Cuenta de Servicios 1627610 Totalapagarhastael13/07/2026 $204.843,80 CUITNº30-70956507-5 LSP 0111B17461407";
        let data = parse_text_fields_excluding(compact, None);
        assert_eq!(data.due_date.as_deref(), Some("2026-07-13"));
        assert_eq!(data.amount, Some(204843.80));
        assert!(data.cuit.as_deref().unwrap_or("").contains("30-70956507"));
        assert!(data.number.is_some());
    }

    #[test]
    fn decode_cid_spaced_factura() {
        let raw = concat!(
            "000F000A000C000T000U000R000A ",
            "000V000E000N000C000I000M000I000E000N000T000O000:000 ",
            "00000001000/00010002000/0002000000020006"
        );
        let flat = normalize_pdf_text(raw);
        assert!(flat.to_ascii_lowercase().contains("factura"), "{flat}");
        assert!(flat.to_ascii_lowercase().contains("vencimiento"), "{flat}");
        assert_eq!(find_due_date(&flat).as_deref(), Some("2026-12-01"));
    }

}

