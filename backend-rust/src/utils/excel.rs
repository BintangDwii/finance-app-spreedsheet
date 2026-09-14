use crate::error::{AppError, Result};

pub type ExportRow = (
    i64,
    String,
    String,
    String,
    String,
    String,
    String,
    f64,
    String,
);

pub fn build_excel(rows: Vec<ExportRow>) -> Result<Vec<u8>> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    let headers = [
        "ID", "Tanggal", "Jenis", "Divisi", "Kategori", "Entitas", "Akun", "Total", "Status",
    ];
    for (col, h) in headers.iter().enumerate() {
        sheet
            .write_string(0, col as u16, *h)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    for (row_idx, (id, tanggal, jenis, divisi, kategori, entitas, akun, total, status)) in
        rows.into_iter().enumerate()
    {
        let excel_row = (row_idx + 1) as u32;
        sheet
            .write_number(excel_row, 0, id as f64)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 1, &tanggal)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 2, &jenis)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 3, &divisi)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 4, &kategori)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 5, &entitas)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 6, &akun)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_number(excel_row, 7, total)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sheet
            .write_string(excel_row, 8, &status)
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_excel_empty() {
        let excel_buf = build_excel(vec![]).unwrap();
        assert!(!excel_buf.is_empty());
    }
}
