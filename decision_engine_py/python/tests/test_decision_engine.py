import numpy as np
import pandas as pd
from decision_engine.engine import DecisionEngine


def test_decision_engine_abstraction():
    new_decision_engine = DecisionEngine.model_validate(
        {
            "config": {
                "key": "action",
                "default_action": "high",
                "rules": [
                    ["score < 80", "low"],
                    ["score <= 99", "medium"],
                    ["score > 99", "high"],
                ],
            },
        }
    )
    assert (
        new_decision_engine.transform(pd.DataFrame({"score": [0.0]}))["action"] == "low"
    ).all()


def test_decision_engine_auto_high():
    new_decision_engine = DecisionEngine.model_validate(
        {
            "config": {
                "key": "action",
                "default_action": "high",
                "rules": [
                    ["is_pep > 0", "auto_high"],
                    ["score < 80", "low"],
                    ["score <= 99", "medium"],
                    ["score > 99", "high"],
                ],
            },
        }
    )
    assert (
        new_decision_engine.transform(pd.DataFrame({"score": [0], "is_pep": [100.0]}))[
            "action"
        ]
        == "auto_high"
    ).all()
    assert (
        new_decision_engine.transform(pd.DataFrame({"score": [100], "is_pep": [0]}))[
            "action"
        ]
        == "high"
    ).all()


def test_decision_engine_multi_score():
    new_decision_engine = DecisionEngine(
        config=dict(
            key="action",
            default_action="high",
            rules=[
                ["model1 + model2 < 50", "low"],
                ["(model1 + model2 >= 50) and model3 > 50", "high"],
                ["(model1 + model2 < 99) and (model1 + model2 >= 50)", "medium"],
                ["model1 + model2 > 99", "high"],
            ],
        ),
    )

    input_df = pd.DataFrame(
        {
            "model1": [0, 25, 25, 50],
            "model2": [40, 40, 40, 50],
            "model3": [100, 100, 40, 40],
        }
    )

    output_df = new_decision_engine.transform(input_df)
    assert np.array_equal(
        np.array(output_df["action"]), np.array(["low", "high", "medium", "high"])
    )


def test_decision_engine_action_from_variable():
    new_decision_engine = DecisionEngine.model_validate(
        {
            "config": {
                "key": "action",
                "default_action": "high_variable",
                "rules": [
                    ["is_pep > 0", "high_variable"],
                    ["score < 80", "low_variable"],
                    ["score <= 99", "medium_variable"],
                    ["score > 99", "high_variable"],
                ],
            },
        }
    )
    assert (
        new_decision_engine.transform(
            pd.DataFrame(
                {
                    "score": [0],
                    "is_pep": [100.0],
                    "high_variable": ["high"],
                    "low_variable": ["low"],
                    "medium_variable": ["medium"],
                }
            )
        )["action"]
        == "high"
    ).all()
    assert (
        new_decision_engine.transform(
            pd.DataFrame(
                {
                    "score": [50],
                    "is_pep": [0],
                    "high_variable": ["high"],
                    "low_variable": ["low"],
                    "medium_variable": ["medium"],
                }
            )
        )["action"]
        == "low"
    ).all()
