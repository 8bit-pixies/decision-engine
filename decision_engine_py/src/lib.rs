use decision_engine::decision_engine::DecisionEngine;

use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use pyo3_polars::PySeries;

#[pyclass(subclass)]
pub(crate) struct DecisionEngineInternal {
    key: String,
    default_action: String,
    rule_definitions: Vec<(String, String)>,
}

#[pymethods]
impl DecisionEngineInternal {
    #[new]
    fn new(key: String, default_action: String, rule_definitions: Vec<(String, String)>) -> Self {
        DecisionEngineInternal {
            key,
            default_action,
            rule_definitions,
        }
    }

    pub fn execute(&self, df: PyDataFrame) -> PyResult<PySeries> {
        let decision_engine = DecisionEngine::new(
            self.key.to_owned(),
            self.default_action.to_owned(),
            self.rule_definitions.to_owned(),
        )
        .unwrap();
        let rust_df = df.into();
        Ok(PySeries(decision_engine.get_actions(rust_df)))
    }
}

#[pymodule]
fn decision_engine_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DecisionEngineInternal>()?;
    Ok(())
}
