use crate::error::{Error, LoadError};

use polars::datatypes::AnyValue::Utf8;
use polars::prelude::*;
// use polars_core::frame::DataFrame;
// use polars_core::series::Series;
// use polars::chunked_array::ops::SortOptions;
// use polars::frame::explode::MeltArgs;
// use polars::prelude::col;
use polars::sql::SQLContext;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path, str};

pub struct DecisionEngine {
    key: String,
    default_action: String,
    possible_actions: Vec<String>,
    rule_definitions: Vec<RuleDefinition>,
}

struct RuleDefinition {
    condition: String,
    action: String,
}

#[derive(Deserialize)]
struct Config {
    key: String,
    rule_config: RuleConfigurations,
}

#[derive(Deserialize)]
struct RuleConfigurations {
    default_action: String,
    rules: Vec<(String, String)>,
}

impl DecisionEngine {
    const TABLE_NAME: &'static str = "temp_decision_engine";
    const DEFAULT_ACTION_COLUMN_NAME: &'static str = "default_action";
    const INDEX_COLUMN_NAME: &'static str = "index";

    pub fn new(
        key: String,
        default_action: String,
        rules: Vec<(String, String)>,
    ) -> Result<Self, Error> {
        let mut rule_definitions = Vec::default();
        let mut possible_actions = Vec::default();
        for (condition, action) in rules {
            rule_definitions.push(RuleDefinition {
                condition: condition,
                action: action.to_owned(),
            });
            possible_actions.push(action.to_owned());
        }
        possible_actions.sort_unstable();
        possible_actions.dedup();

        Ok(Self {
            key,
            default_action,
            possible_actions,
            rule_definitions,
        })
    }

    fn load_from_config(config: Config) -> Result<Self, Error> {
        Self::new(
            config.key,
            config.rule_config.default_action,
            config.rule_config.rules,
        )
    }

    pub fn load(config_path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_path = config_path.as_ref().canonicalize().unwrap();
        let config = Config::load(&config_path)?;
        Self::load_from_config(config)
    }

    pub fn load_from_json_string(config_as_json: &str) -> Result<Self, Error> {
        let config = Config::load_from_json_string(&config_as_json)?;
        Self::load_from_config(config)
    }

    fn execute_decision_engine(&self, features_dataframe: DataFrame) -> Result<Vec<String>, Error> {
        let mut context = SQLContext::new();
        context.register(
            DecisionEngine::TABLE_NAME,
            features_dataframe.clone().lazy(),
        );

        let mut select_statement: String = DecisionEngine::INDEX_COLUMN_NAME.to_string();
        let mut action_outcome: Vec<String> = Vec::default();

        for (indx, rule_definition) in self.rule_definitions.iter().enumerate() {
            let case_when_string: &str = &format!(
                "\n,{condition} as {condition_name}",
                condition = rule_definition.condition.clone(),
                condition_name = format!("condition_{}", indx),
            );
            select_statement.push_str(case_when_string);
            action_outcome.push(rule_definition.action.clone());
        }
        action_outcome.push(self.default_action.clone());
        select_statement.push_str(&format!(
            ",true as {}",
            DecisionEngine::DEFAULT_ACTION_COLUMN_NAME
        ));

        let sql_query: String = format!(
            "SELECT {}\nFROM {}\n",
            select_statement,
            DecisionEngine::TABLE_NAME
        );
        let decision_dataframe = context.execute(&sql_query).unwrap().collect().unwrap();

        // melt by index
        let mut melted_decision_dataframe = decision_dataframe
            .melt2(MeltArgs {
                id_vars: vec![DecisionEngine::INDEX_COLUMN_NAME.to_string().into()],
                value_vars: Vec::default(),
                variable_name: None,
                value_name: None,
                streamable: false,
            })
            .unwrap()
            .sort(&["index", "variable"], false, false)
            .unwrap();

        // add actions - as dataframe is sorted, cycling through will work
        let action_repeated: Vec<_> = action_outcome
            .iter()
            .cycle()
            .take(melted_decision_dataframe.height())
            .map(|x| x.to_string())
            .collect();
        let action_column = Series::new("actions", action_repeated);
        melted_decision_dataframe.with_column(action_column).ok();

        let executed_decision = melted_decision_dataframe
            .lazy()
            .filter(col("value"))
            .sort("variable", SortOptions::default())
            .groupby([col(DecisionEngine::INDEX_COLUMN_NAME)])
            .agg([col("actions").first()])
            .sort(
                "index",
                SortOptions {
                    descending: false,
                    nulls_last: false,
                    multithreaded: true,
                    maintain_order: false,
                },
            )
            .collect()
            .unwrap();

        // select the column and return
        Ok(executed_decision
            .column("actions")
            .unwrap()
            .utf8()
            .unwrap()
            .into_no_null_iter()
            .map(|value| value.to_owned())
            .collect())
    }

    // return the evaluation for all rules
    pub fn get_actions(&self, df: DataFrame) -> Series {
        let mut dataframe = df.to_owned();
        let n_rows = dataframe.height();
        let index_column = Series::new(
            "index",
            (0..(n_rows as i32)).into_iter().collect::<Vec<i32>>(),
        );
        dataframe.with_column(index_column).ok();
        for action in &self.possible_actions {
            if !dataframe.get_column_names().iter().any(|&v| v == action) {
                dataframe
                    .with_column(Series::new(action, vec![action.to_string(); n_rows]))
                    .ok();
            }
        }

        let actions = self.execute_decision_engine(dataframe.clone()).unwrap();
        // map things back
        let feature_names = dataframe.get_column_names().to_owned();
        let mut action_values = Vec::<String>::new();
        for row_index in 0..n_rows {
            let action = actions[row_index].as_str();
            if feature_names.iter().any(|&v| v == action) {
                let raw_action = dataframe.column(action).unwrap().get(row_index).unwrap();
                let v = if let Utf8(v) = raw_action {
                    v.to_string()
                } else {
                    raw_action.to_string()
                };
                action_values.push(v.to_string());
            } else {
                action_values.push(action.to_string());
            }
        }

        Series::new(&self.key, action_values)
    }
}

impl Config {
    fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref().canonicalize()?;
        let contents = read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    fn load_from_json_string(json_string: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(json_string).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, DecisionEngine};
    use crate::error::Error;
    use crate::test_utils::testdata_dir;
    use polars::prelude::*;
    use std::path::PathBuf;
    use test_case::test_case;

    fn config_path(base_file_name: &str) -> PathBuf {
        let file_name = format!("{base_file_name}.toml");
        testdata_dir().join(file_name)
    }

    #[test_case("simple_rules")]
    fn load_config(base_file_name: &str) -> Result<(), Error> {
        let file_path = config_path(base_file_name);
        let _config = Config::load(file_path)?;
        Ok(())
    }

    #[test_case("simple_rules")]
    fn load_decision_engine_configuration(base_file_name: &str) -> Result<(), Error> {
        let file_path = config_path(base_file_name);
        let _decision_engine = DecisionEngine::load(file_path)?;
        Ok(())
    }

    #[test_case("simple_rules")]
    fn run_all_scores(base_file_name: &str) -> Result<(), Error> {
        let file_path = config_path(base_file_name);
        let decision_engine = DecisionEngine::load(file_path)?;
        let df = DataFrame::new(vec![Series::new("score", &[0, 50, 85, 95, 105])]).unwrap();
        let actions = decision_engine.get_actions(df);
        assert_eq!(
            actions,
            Series::new(
                &decision_engine.key.to_owned(),
                vec!["low", "low", "medium", "medium", "high"]
            )
        );
        Ok(())
    }
}
