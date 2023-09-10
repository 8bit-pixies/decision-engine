# Decision Engine

> :warning: this is alpha software and there are known bugs due to decision engine trying to dynamically evaluate things

A configuration driven decision engine written in Rust with Python bindings. 

Python usage:

```py
import pandas as pd
from decision_engine.engine import DecisionEngine

decision_engine = DecisionEngine.model_validate(
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


dataframe = pd.DataFrame({"score": [0.0, 85, 100]})
print(decision_engine.transform(dataframe))
#    action
# 0     low
# 1  medium
# 2    high
```

This also works with multiple conditions, whereby it takes the first one which is "true". 

```py
import pandas as pd
from decision_engine.engine import DecisionEngine

decision_engine = DecisionEngine.model_validate(
    {
        "config": {
            "key": "action",
            "default_action": "high",
            "rules": [
                ["override", "'override'"],
                ["not override and score < 100", "low"],
                ["score >= 100", "high"],
            ],
        },
    }
)


dataframe = pd.DataFrame({"score": [0.0, 85, 100], "override": [True, False, False]})
print(decision_engine.transform(dataframe))
#         action
#  0  'override'
#  1         low
#  2        high
```

## Developer Notes

From a development perspective the Rust and Python bindings are kept in two separate crates:

* `decision_engine`
* `decision_engine_py`

This allows the Rust crate to remain "pure" and the Python bindings to be developed separately. 

The decision engine is configuration driven which can easily be exported in `toml` or `json` format and parsed in Rust or Python. 

For example, in Rust, we may define a decision engine as follows:

```toml
key = "action"

[rule_config]
default_action = "'high'"
rules = [
    ["score <= 80", "low"],
    ["score <=100", "medium"],
    ["score > 100", "high"],
]
```

Which would have the corresponding `json` setup as

```json
{
  "config": {
    "key": "action",
    "default_action": "high",
    "rules": [
      [
        "score < 80",
        "low"
      ],
      [
        "score <= 99",
        "medium"
      ],
      [
        "score > 99",
        "high"
      ]
    ]
  }
}
```

### Tests and Formatting

We use `just` for running tests and auto formatters. This can be installed via `brew install just`

```sh
$ just test
$ just format
$ just lint
```
