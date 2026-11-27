use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{AddPaymentPartial, Payment, PaymentEvent, UpdatePayment};
use crate::state::AppState;
use crate::util::{new_id, now_iso};
use rusqlite::{params, Connection};
use tauri::State;

const PAYMENT_SELECT: &str = "
    SELECT pay.id, pay.document_id, pay.entity_id, pay.status, pay.amount_due,
           pay.amount_paid, pay.paid_at, pay.notes, pay.updated_at,
           d.file_name, p.name, d.due_date, d.number, d.issue_date
    FROM payments pay
    JOIN documents d ON d.id = pay.document_id AND d.entity_id = pay.entity_id
    LEFT JOIN providers p ON p.id = d.provider_id AND p.entity_id = pay.entity_id
";

fn map_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get(0)?,
        document_id: row.get(1)?,
        entity_id: row.get(2)?,
        status: row.get(3)?,
        amount_due: row.get(4)?,
        amount_paid: row.get(5)?,
        paid_at: row.get(6)?,
        notes: row.get(7)?,
        updated_at: row.get(8)?,
        file_name: row.get(9)?,
        provider_name: row.get(10)?,
        due_date: row.get(11)?,
        number: row.get(12)?,
        issue_date: row.get(13)?,
    })
}

fn fetch_payment(
    conn: &Connection,
    entity_id: &str,
    document_id: &str,
) -> AppResult<Payment> {
    let sql = format!("{PAYMENT_SELECT} WHERE pay.document_id = ?1 AND pay.entity_id = ?2");
    conn.query_row(&sql, params![document_id, entity_id], map_payment)
        .map_err(|_| AppError::msg("Pago no encontrado"))
}

fn payment_id_for_document(
    conn: &Connection,
    entity_id: &str,
    document_id: &str,
) -> AppResult<(String, Option<f64>)> {
    conn.query_row(
        "SELECT id, amount_due FROM payments WHERE document_id = ?1 AND entity_id = ?2",
        params![document_id, entity_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|_| AppError::msg("Pago no encontrado para el documento"))
}

fn sum_events(conn: &Connection, payment_id: &str) -> AppResult<f64> {
    let total: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM payment_events WHERE payment_id = ?1",
        params![payment_id],
        |row| row.get(0),
    )?;
    Ok(total)
}

fn today_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn resolve_status(amount_paid: f64, amount_due: Option<f64>, preferred: &str) -> &'static str {
    if preferred == "pending" {
        return "pending";
    }
    if let Some(due) = amount_due {
        if due > 0.0 && amount_paid + 0.000_1 >= due {
            return "paid";
        }
    }
    if preferred == "paid" {
        return "paid";
    }
    if amount_paid > 0.0 {
        "partial"
    } else {
        "pending"
    }
}

#[tauri::command]
pub fn list_payments(
    entity_id: String,
    status: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Payment>> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let mut sql = format!("{PAYMENT_SELECT} WHERE pay.entity_id = ?1");
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(entity_id)];
        if let Some(st) = status
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND pay.status = ?");
            values.push(Box::new(st));
        }
        sql.push_str(" ORDER BY IFNULL(d.due_date, '9999') ASC, pay.updated_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(values.iter().map(|v| &**v)), map_payment)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })
}

#[tauri::command]
pub fn update_payment(payload: UpdatePayment, state: State<'_, AppState>) -> AppResult<Payment> {
    let status = match payload.status.as_str() {
        "pending" | "partial" | "paid" => payload.status.as_str(),
        _ => return Err(AppError::msg("Estado de pago inválido")),
    };
    let updated_at = now_iso();
    let today = today_date();

    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &payload.entity_id, &payload.document_id)?;
        let (payment_id, amount_due_db) =
            payment_id_for_document(conn, &payload.entity_id, &payload.document_id)?;
        let amount_due = payload.amount_due.or(amount_due_db);

        match status {
            "pending" => {
                conn.execute(
                    "DELETE FROM payment_events WHERE payment_id = ?1 AND entity_id = ?2",
                    params![payment_id, payload.entity_id],
                )?;
                conn.execute(
                    "UPDATE payments SET
                        status = 'pending',
                        amount_due = COALESCE(?1, amount_due),
                        amount_paid = 0,
                        paid_at = NULL,
                        notes = COALESCE(?2, notes),
                        updated_at = ?3
                     WHERE id = ?4 AND entity_id = ?5",
                    params![
                        amount_due,
                        payload.notes,
                        updated_at,
                        payment_id,
                        payload.entity_id
                    ],
                )?;
            }
            "paid" => {
                let current_paid = sum_events(conn, &payment_id)?;
                if let Some(due) = amount_due {
                    let remaining = due - current_paid;
                    if remaining > 0.000_1 {
                        let event_id = new_id();
                        conn.execute(
                            "INSERT INTO payment_events
                                (id, payment_id, entity_id, amount, paid_at, note, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            params![
                                event_id,
                                payment_id,
                                payload.entity_id,
                                remaining,
                                payload.paid_at.clone().unwrap_or_else(|| today.clone()),
                                "Saldo al marcar pagado",
                                updated_at
                            ],
                        )?;
                    }
                }
                let paid_total = sum_events(conn, &payment_id)?;
                let final_paid = payload
                    .amount_paid
                    .unwrap_or(paid_total.max(amount_due.unwrap_or(paid_total)));
                conn.execute(
                    "UPDATE payments SET
                        status = 'paid',
                        amount_due = COALESCE(?1, amount_due),
                        amount_paid = ?2,
                        paid_at = COALESCE(?3, paid_at, ?4),
                        notes = COALESCE(?5, notes),
                        updated_at = ?6
                     WHERE id = ?7 AND entity_id = ?8",
                    params![
                        amount_due,
                        final_paid,
                        payload.paid_at,
                        today,
                        payload.notes,
                        updated_at,
                        payment_id,
                        payload.entity_id
                    ],
                )?;
            }
            "partial" => {
                let paid_total = sum_events(conn, &payment_id)?;
                let amount_paid = payload.amount_paid.unwrap_or(paid_total);
                let next = resolve_status(amount_paid, amount_due, "partial");
                conn.execute(
                    "UPDATE payments SET
                        status = ?1,
                        amount_due = COALESCE(?2, amount_due),
                        amount_paid = ?3,
                        paid_at = CASE WHEN ?1 = 'paid' THEN COALESCE(paid_at, ?4) ELSE paid_at END,
                        notes = COALESCE(?5, notes),
                        updated_at = ?6
                     WHERE id = ?7 AND entity_id = ?8",
                    params![
                        next,
                        amount_due,
                        amount_paid,
                        today,
                        payload.notes,
                        updated_at,
                        payment_id,
                        payload.entity_id
                    ],
                )?;
            }
            _ => unreachable!(),
        }
        Ok(())
    })?;

    state.with_db(|conn| fetch_payment(conn, &payload.entity_id, &payload.document_id))
}

#[tauri::command]
pub fn list_payment_events(
    entity_id: String,
    document_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<PaymentEvent>> {
    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &document_id)?;
        let (payment_id, _) = payment_id_for_document(conn, &entity_id, &document_id)?;
        let mut stmt = conn.prepare(
            "
            SELECT id, payment_id, entity_id, amount, paid_at, note, created_at
            FROM payment_events
            WHERE payment_id = ?1 AND entity_id = ?2
            ORDER BY paid_at ASC, created_at ASC
            ",
        )?;
        let rows = stmt.query_map(params![payment_id, entity_id], |row| {
            Ok(PaymentEvent {
                id: row.get(0)?,
                payment_id: row.get(1)?,
                entity_id: row.get(2)?,
                amount: row.get(3)?,
                paid_at: row.get(4)?,
                note: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })
}

#[tauri::command]
pub fn add_payment_partial(
    payload: AddPaymentPartial,
    state: State<'_, AppState>,
) -> AppResult<Payment> {
    if payload.amount <= 0.0 {
        return Err(AppError::msg("El monto del pago parcial debe ser mayor a 0"));
    }
    let updated_at = now_iso();
    let paid_at = payload
        .paid_at
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(today_date);

    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &payload.entity_id, &payload.document_id)?;
        let (payment_id, amount_due) =
            payment_id_for_document(conn, &payload.entity_id, &payload.document_id)?;
        let event_id = new_id();
        conn.execute(
            "INSERT INTO payment_events
                (id, payment_id, entity_id, amount, paid_at, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event_id,
                payment_id,
                payload.entity_id,
                payload.amount,
                paid_at,
                payload.note,
                updated_at
            ],
        )?;
        let paid_total = sum_events(conn, &payment_id)?;
        let next = resolve_status(paid_total, amount_due, "partial");
        conn.execute(
            "UPDATE payments SET
                status = ?1,
                amount_paid = ?2,
                paid_at = CASE WHEN ?1 = 'paid' THEN COALESCE(paid_at, ?3) ELSE paid_at END,
                updated_at = ?4
             WHERE id = ?5 AND entity_id = ?6",
            params![
                next,
                paid_total,
                paid_at,
                updated_at,
                payment_id,
                payload.entity_id
            ],
        )?;
        Ok(())
    })?;

    state.with_db(|conn| fetch_payment(conn, &payload.entity_id, &payload.document_id))
}

#[tauri::command]
pub fn delete_payment_event(
    entity_id: String,
    document_id: String,
    event_id: String,
    state: State<'_, AppState>,
) -> AppResult<Payment> {
    let updated_at = now_iso();
    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &document_id)?;
        let (payment_id, amount_due) = payment_id_for_document(conn, &entity_id, &document_id)?;
        let changed = conn.execute(
            "DELETE FROM payment_events
             WHERE id = ?1 AND payment_id = ?2 AND entity_id = ?3",
            params![event_id, payment_id, entity_id],
        )?;
        if changed == 0 {
            return Err(AppError::msg("Pago parcial no encontrado"));
        }
        let paid_total = sum_events(conn, &payment_id)?;
        let next = resolve_status(paid_total, amount_due, if paid_total > 0.0 { "partial" } else { "pending" });
        conn.execute(
            "UPDATE payments SET
                status = ?1,
                amount_paid = ?2,
                paid_at = CASE WHEN ?1 = 'paid' THEN COALESCE(paid_at, ?3) ELSE NULL END,
                updated_at = ?4
             WHERE id = ?5 AND entity_id = ?6",
            params![next, paid_total, today_date(), updated_at, payment_id, entity_id],
        )?;
        Ok(())
    })?;
    state.with_db(|conn| fetch_payment(conn, &entity_id, &document_id))
}

#[tauri::command]
pub fn list_agenda(entity_id: String, state: State<'_, AppState>) -> AppResult<Vec<Payment>> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let sql = format!(
            "{PAYMENT_SELECT}
             WHERE pay.entity_id = ?1
               AND pay.status IN ('pending', 'partial')
             ORDER BY IFNULL(d.due_date, '9999') ASC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![entity_id], map_payment)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })
}
