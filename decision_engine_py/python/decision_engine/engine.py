from typing import List, Tuple

import pandas as pd
import polars as pl
from pydantic import BaseModel, field_validator

from decision_engine import DecisionEngineInternal


class Rule(BaseModel):
    condition: str
    action: str

    def to_list(self):
        return [self.condition, self.action]


class DecisionEngineConfig(BaseModel):
    key: str
    default_action: str
    rules: List[Tuple[str, str]]

    @field_validator("rules", mode="before")
    def update_rules_from_iterable(cls, values):
        for v in values:
            assert (
                len(v) == 2
            ), f"If rules is an iterable it has to be len() == 2. Got {v}"
        return values


class DecisionEngine(BaseModel):
    config: DecisionEngineConfig

    def __init__(
        self,
        config: DecisionEngineConfig,
    ):
        super().__init__(id=id, config=config)
        self._decision_engine = self._create_decision_engine()

    def _create_decision_engine(self):
        rule_definitions = [tuple(x) for x in self.config.rules]
        decision_engine = DecisionEngineInternal(
            self.config.key, self.config.default_action, rule_definitions
        )
        return decision_engine

    def transform(self, df: pd.DataFrame):
        return self._decision_engine.execute(pl.from_pandas(df)).to_frame().to_pandas()
