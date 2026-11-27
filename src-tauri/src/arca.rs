use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD},
    Engine as _,
};
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct InvoiceData {
    pub cuit: Option<String>,
    pub provider_name: Option<String>,
    pub number: Option<String>,
    pub point_of_sale: Option<i64>,
    pub period: Option<String>,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub fx_rate: Option<f64>,
    pub doc_type: Option<String>,
    pub auth_type: Option<String>,
    pub auth_code: Option<String>,
    pub receptor_doc_type: Option<i64>,
    pub receptor_doc_number: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AfipQrPayload {
    #[serde(default)]
    ver: Option<i64>,
    #[serde(default)]
    fecha: Option<String>,
    #[serde(default)]
    cuit: Option<serde_json::Value>,
    #[serde(rename = "ptoVta", default)]
    pto_vta: Option<i64>,
    #[serde(rename = "tipoCmp", default)]
    tipo_cmp: Option<i64>,
    #[serde(rename = "nroCmp", default)]
    nro_cmp: Option<i64>,
    #[serde(default)]
    importe: Option<f64>,
    #[serde(default)]
    moneda: Option<String>,
    #[serde(default)]
    ctz: Option<f64>,
    #[serde(rename = "tipoDocRec", default)]
    tipo_doc_rec: Option<i64>,
    #[serde(rename = "nroDocRec", default)]
    nro_doc_rec: Option<serde_json::Value>,
    #[serde(rename = "tipoCodAut", default)]
    tipo_cod_aut: Option<String>,
    #[serde(rename = "codAut", default)]
    cod_aut: Option<serde_json::Value>,
}

pub fn parse_arca_qr_content(content: &str) -> Option<InvoiceData> {
    let payload_b64 = extract_p_param(content).unwrap_or(content.trim());
    let trimmed = payload_b64.trim();
    let decoded = STANDARD
        .decode(trimmed)
        .or_else(|_| URL_SAFE.decode(trimmed))
        .or_else(|_| URL_SAFE_NO_PAD.decode(trimmed))
        .ok()?;
    let json = String::from_utf8_lossy(&decoded);
    let payload: AfipQrPayload = serde_json::from_str(&json).ok()?;

    if let Some(ver) = payload.ver {
        if ver != 1 {
            log::warn!("QR ARCA versión inesperada: {ver}");
        }
    }

    let cuit = payload.cuit.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64().map(|i| format_cuit(&i.to_string())),
        serde_json::Value::String(s) => Some(format_cuit(&s)),
        _ => None,
    });

    let issue_date = payload.fecha.map(|f| normalize_date(&f));
    let period = issue_date.as_ref().and_then(|d| {
        if d.len() >= 7 {
            Some(d[..7].to_string())
        } else {
            None
        }
    });

    let point_of_sale = payload.pto_vta;
    let number = match (payload.pto_vta, payload.nro_cmp) {
        (Some(pto), Some(nro)) => Some(format!("{pto:04}-{nro:08}")),
        (_, Some(nro)) => Some(format!("{nro:08}")),
        _ => None,
    };

    let doc_type = payload.tipo_cmp.map(tipo_comprobante_label);
    let currency = payload.moneda.map(|m| {
        if m.eq_ignore_ascii_case("PES") || m == "ARS" {
            "ARS".into()
        } else {
            m
        }
    });

    let auth_type = payload.tipo_cod_aut.map(|t| match t.as_str() {
        "E" => "CAE".into(),
        "A" => "CAEA".into(),
        other => other.to_string(),
    });

    let auth_code = payload.cod_aut.and_then(|v| match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|u| u.to_string())
            .or_else(|| n.as_i64().map(|i| i.to_string())),
        serde_json::Value::String(s) => Some(s),
        _ => None,
    });

    let receptor_doc_number = payload.nro_doc_rec.and_then(|v| match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|u| u.to_string())
            .or_else(|| n.as_i64().map(|i| i.to_string())),
        serde_json::Value::String(s) => Some(s),
        _ => None,
    });

    log::info!(
        "QR ARCA parseado: pto={:?} nro={:?} moneda={:?} auth={:?}",
        point_of_sale,
        number,
        currency,
        auth_type
    );

    Some(InvoiceData {
        cuit,
        provider_name: None,
        number,
        point_of_sale,
        period,
        issue_date,
        due_date: None,
        amount: payload.importe,
        currency,
        fx_rate: payload.ctz,
        doc_type,
        auth_type,
        auth_code,
        receptor_doc_type: payload.tipo_doc_rec,
        receptor_doc_number,
        source: Some("qr_arca".into()),
    })
}

fn extract_p_param(content: &str) -> Option<&str> {
    if let Some(idx) = content.find("?p=") {
        let rest = &content[idx + 3..];
        let end = rest
            .find(|c: char| {
                c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == '>' || c == '&'
            })
            .unwrap_or(rest.len());
        return Some(&rest[..end]);
    }
    if let Some(idx) = content.find("&p=") {
        let rest = &content[idx + 3..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == '&')
            .unwrap_or(rest.len());
        return Some(&rest[..end]);
    }
    None
}

pub fn format_cuit(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        format!("{}-{}-{}", &digits[0..2], &digits[2..10], &digits[10..11])
    } else {
        digits
    }
}

pub fn normalize_cuit_key(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn normalize_date(value: &str) -> String {
    if value.contains('/') {
        let parts: Vec<_> = value.split('/').collect();
        if parts.len() == 3 {
            return format!("{}-{}-{}", parts[2], parts[1], parts[0]);
        }
    }
    value.to_string()
}

fn tipo_comprobante_label(tipo: i64) -> String {
    match tipo {
        1 => "Factura A".into(),
        6 => "Factura B".into(),
        11 => "Factura C".into(),
        3 => "Nota de Crédito A".into(),
        8 => "Nota de Crédito B".into(),
        13 => "Nota de Crédito C".into(),
        2 => "Nota de Débito A".into(),
        7 => "Nota de Débito B".into(),
        12 => "Nota de Débito C".into(),
        51 => "Factura M".into(),
        other => format!("Comprobante {other}"),
    }
}

pub fn merge_invoice_data(base: &mut InvoiceData, extra: InvoiceData) {
    if base.cuit.is_none() {
        base.cuit = extra.cuit;
    }
    if base.provider_name.is_none() {
        base.provider_name = extra.provider_name;
    }
    if base.number.is_none() {
        base.number = extra.number;
    }
    if base.point_of_sale.is_none() {
        base.point_of_sale = extra.point_of_sale;
    }
    if base.period.is_none() {
        base.period = extra.period;
    }
    if base.issue_date.is_none() {
        base.issue_date = extra.issue_date;
    }
    if base.due_date.is_none() {
        base.due_date = extra.due_date;
    }
    if base.amount.is_none() {
        base.amount = extra.amount;
    }
    if base.currency.is_none() {
        base.currency = extra.currency;
    }
    if base.fx_rate.is_none() {
        base.fx_rate = extra.fx_rate;
    }
    if base.doc_type.is_none() {
        base.doc_type = extra.doc_type;
    }
    if base.auth_type.is_none() {
        base.auth_type = extra.auth_type;
    }
    if base.auth_code.is_none() {
        base.auth_code = extra.auth_code;
    }
    if base.receptor_doc_type.is_none() {
        base.receptor_doc_type = extra.receptor_doc_type;
    }
    if base.receptor_doc_number.is_none() {
        base.receptor_doc_number = extra.receptor_doc_number;
    }
    if base.source.is_none() {
        base.source = extra.source;
    } else if let Some(src) = extra.source {
        base.source = Some(format!("{},{}", base.source.clone().unwrap_or_default(), src));
    }
}
