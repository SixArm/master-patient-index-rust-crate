//! Search index management with Tantivy

use tantivy::{
    schema::{Schema, Field, STORED, TEXT, STRING, FAST},
    Index, IndexWriter, IndexReader, ReloadPolicy,
    collector::TopDocs,
    query::QueryParser,
    doc,
};
use std::path::Path;
use uuid::Uuid;

use crate::Result;

/// Fields in the patient search index
#[derive(Clone)]
pub struct PatientIndexSchema {
    pub schema: Schema,
    pub id: Field,
    pub family_name: Field,
    pub given_names: Field,
    pub full_name: Field,
    pub birth_date: Field,
    pub gender: Field,
    pub postal_code: Field,
    pub city: Field,
    pub state: Field,
    pub identifiers: Field,
    pub active: Field,
}

impl PatientIndexSchema {
    /// Create the patient index schema
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        // ID field (stored, not indexed for search)
        let id = schema_builder.add_text_field("id", STRING | STORED);

        // Name fields (indexed and stored)
        let family_name = schema_builder.add_text_field("family_name", TEXT | STORED);
        let given_names = schema_builder.add_text_field("given_names", TEXT | STORED);
        let full_name = schema_builder.add_text_field("full_name", TEXT | STORED);

        // Demographics (indexed and stored)
        let birth_date = schema_builder.add_text_field("birth_date", STRING | STORED);
        let gender = schema_builder.add_text_field("gender", STRING | STORED);

        // Address fields (indexed and stored)
        let postal_code = schema_builder.add_text_field("postal_code", STRING | STORED);
        let city = schema_builder.add_text_field("city", TEXT | STORED);
        let state = schema_builder.add_text_field("state", STRING | STORED);

        // Identifiers (indexed and stored)
        let identifiers = schema_builder.add_text_field("identifiers", TEXT | STORED);

        // Active status (for filtering)
        let active = schema_builder.add_text_field("active", STRING | FAST);

        let schema = schema_builder.build();

        Self {
            schema,
            id,
            family_name,
            given_names,
            full_name,
            birth_date,
            gender,
            postal_code,
            city,
            state,
            identifiers,
            active,
        }
    }
}

impl Default for PatientIndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Patient search index
pub struct PatientIndex {
    index: Index,
    schema: PatientIndexSchema,
    reader: IndexReader,
}

impl PatientIndex {
    /// Create a new index at the given path
    pub fn create<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let schema_def = PatientIndexSchema::new();
        let index = Index::create_in_dir(index_path, schema_def.schema.clone())
            .map_err(|e| crate::Error::Search(format!("Failed to create index: {}", e)))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| crate::Error::Search(format!("Failed to create reader: {}", e)))?;

        Ok(Self {
            index,
            schema: schema_def,
            reader,
        })
    }

    /// Open an existing index at the given path
    pub fn open<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let schema_def = PatientIndexSchema::new();
        let index = Index::open_in_dir(index_path)
            .map_err(|e| crate::Error::Search(format!("Failed to open index: {}", e)))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| crate::Error::Search(format!("Failed to create reader: {}", e)))?;

        Ok(Self {
            index,
            schema: schema_def,
            reader,
        })
    }

    /// Create or open an index
    pub fn create_or_open<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let path = index_path.as_ref();
        let meta_path = path.join("meta.json");

        if meta_path.exists() {
            Self::open(index_path)
        } else {
            Self::create(index_path)
        }
    }

    /// Get an index writer
    pub fn writer(&self, heap_size_mb: usize) -> Result<IndexWriter> {
        self.index
            .writer(heap_size_mb * 1_000_000)
            .map_err(|e| crate::Error::Search(format!("Failed to create writer: {}", e)))
    }

    /// Get the index
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Get the schema
    pub fn schema(&self) -> &PatientIndexSchema {
        &self.schema
    }

    /// Get the reader
    pub fn reader(&self) -> &IndexReader {
        &self.reader
    }

    /// Manually reload the reader (useful for tests)
    pub fn reload(&self) -> Result<()> {
        self.reader.reload()
            .map_err(|e| crate::Error::Search(format!("Failed to reload reader: {}", e)))
    }

    /// Get index statistics
    pub fn stats(&self) -> Result<IndexStats> {
        let searcher = self.reader.searcher();
        let num_docs = searcher.num_docs() as usize;
        let num_segments = searcher.segment_readers().len();

        Ok(IndexStats {
            num_docs,
            num_segments,
        })
    }

    /// Optimize the index (wait for merges to complete)
    pub fn optimize(&self) -> Result<()> {
        let mut writer = self.writer(50)?;
        writer
            .wait_merging_threads()
            .map_err(|e| crate::Error::Search(format!("Failed to optimize index: {}", e)))?;
        Ok(())
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub num_docs: usize,
    pub num_segments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_index() {
        let temp_dir = TempDir::new().unwrap();
        let index = PatientIndex::create(temp_dir.path()).unwrap();

        let stats = index.stats().unwrap();
        assert_eq!(stats.num_docs, 0);
    }

    #[test]
    fn test_schema_fields() {
        let schema = PatientIndexSchema::new();

        // Verify fields exist
        let _ = schema.id;
        let _ = schema.family_name;
        let _ = schema.given_names;
        let _ = schema.full_name;
        let _ = schema.birth_date;
        let _ = schema.gender;
    }

    #[test]
    fn test_create_or_open() {
        let temp_dir = TempDir::new().unwrap();

        // First call creates
        let index1 = PatientIndex::create_or_open(temp_dir.path()).unwrap();
        assert_eq!(index1.stats().unwrap().num_docs, 0);

        // Second call opens
        let index2 = PatientIndex::create_or_open(temp_dir.path()).unwrap();
        assert_eq!(index2.stats().unwrap().num_docs, 0);
    }
}
