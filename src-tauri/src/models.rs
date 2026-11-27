use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub tax_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEntity {
    pub name: String,
    pub kind: String,
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub cuit: Option<String>,
    pub alias: Option<String>,
    pub cbu: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProvider {
    pub entity_id: String,
    pub name: String,
    pub cuit: Option<String>,
    pub alias: Option<String>,
    pub cbu: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProvider {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub cuit: Option<String>,
    pub alias: Option<String>,
    pub cbu: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub entity_id: String,
    pub provider_id: Option<String>,
    pub file_name: String,
    pub original_path: Option<String>,
    pub stored_path: String,
    pub file_hash: String,
    pub mime_type: Option<String>,
    pub doc_type: Option<String>,
    pub number: Option<String>,
    pub period: Option<String>,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub amount: Option<f64>,
    pub currency: String,
    pub point_of_sale: Option<i64>,
    pub fx_rate: Option<f64>,
    pub auth_type: Option<String>,
    pub auth_code: Option<String>,
    pub receptor_doc_type: Option<i64>,
    pub receptor_doc_number: Option<String>,
    pub notes: Option<String>,
    pub imported_at: String,
    pub provider_name: Option<String>,
    pub payment_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilters {
    pub entity_id: String,
    pub query: Option<String>,
    pub provider: Option<String>,
    pub cuit: Option<String>,
    pub number: Option<String>,
    pub period: Option<String>,
    pub payment_status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub documents: Vec<Document>,
    pub total_amount: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: String,
    pub document_id: String,
    pub entity_id: String,
    pub status: String,
    pub amount_due: Option<f64>,
    pub amount_paid: f64,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
    pub updated_at: String,
    pub file_name: Option<String>,
    pub provider_name: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePayment {
    pub entity_id: String,
    pub document_id: String,
    pub status: String,
    pub amount_due: Option<f64>,
    pub amount_paid: Option<f64>,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub imported: usize,
    pub duplicates: usize,
    pub errors: Vec<String>,
    pub document_ids: Vec<String>,
    pub qr_parsed: usize,
    pub ocr_used: usize,
    pub providers_created: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub data_dir: String,
    pub db_path: String,
    pub documents_dir: String,
    pub backups_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub entities: usize,
    pub documents: usize,
    pub providers: usize,
    pub pending_payments: usize,
    pub due_soon: usize,
}
