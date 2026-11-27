use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Payment, UpdatePayment};
use crate::state::AppState;
use crate::util::now_iso;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_payments(
    entity_id: String,
    status: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Payment>> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let mut sql = String::from(
            "
            SELECT pay.id, pay.document_id, pay.entity_id, pay.status, pay.amount_due,
                   pay.amount_paid, pay.paid_at, pay.notes, pay.updated_at,
                   d.file_name, p.name, d.due_date
            FROM payments pay
            JOIN documents d ON d.id = pay.document_id
            LEFT JOIN providers p ON p.id = d.provider_id
            WHERE pay.entity_id = ?1
            ",
        );
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
        let rows = stmt.query_map(rusqlite::params_from_iter(values.iter().map(|v| &**v)), |row| {
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
pub fn update_payment(payload: UpdatePayment, state: State<'_, AppState>) -> AppResult<Payment> {
    let status = match payload.status.as_str() {
        "pending" | "partial" | "paid" => payload.status.as_str(),
        _ => return Err(AppError::msg("Estado de pago inválido")),
    };
    let updated_at = now_iso();

    state.with_db(|conn| {
        let changed = conn.execute(
            "UPDATE payments SET
                status = ?1,
                amount_due = COALESCE(?2, amount_due),
                amount_paid = COALESCE(?3, amount_paid),
                paid_at = ?4,
                notes = COALESCE(?5, notes),
                updated_at = ?6
             WHERE document_id = ?7",
            params![
                status,
                payload.amount_due,
                payload.amount_paid,
                payload.paid_at,
                payload.notes,
                updated_at,
                payload.document_id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::msg("Pago no encontrado para el documento"));
        }
        Ok(())
    })?;

    state.with_db(|conn| {
        conn.query_row(
            "
            SELECT pay.id, pay.document_id, pay.entity_id, pay.status, pay.amount_due,
                   pay.amount_paid, pay.paid_at, pay.notes, pay.updated_at,
                   d.file_name, p.name, d.due_date
            FROM payments pay
            JOIN documents d ON d.id = pay.document_id
            LEFT JOIN providers p ON p.id = d.provider_id
            WHERE pay.document_id = ?1
            ",
            params![payload.document_id],
            |row| {
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
                })
            },
        )
        .map_err(|_| AppError::msg("Pago no encontrado"))
    })
}

#[tauri::command]
pub fn list_agenda(entity_id: String, state: State<'_, AppState>) -> AppResult<Vec<Payment>> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let mut stmt = conn.prepare(
            "
            SELECT pay.id, pay.document_id, pay.entity_id, pay.status, pay.amount_due,
                   pay.amount_paid, pay.paid_at, pay.notes, pay.updated_at,
                   d.file_name, p.name, d.due_date
            FROM payments pay
            JOIN documents d ON d.id = pay.document_id
            LEFT JOIN providers p ON p.id = d.provider_id
            WHERE pay.entity_id = ?1
              AND pay.status IN ('pending', 'partial')
            ORDER BY IFNULL(d.due_date, '9999') ASC
            ",
        )?;
        let rows = stmt.query_map(params![entity_id], |row| {
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
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })
}
