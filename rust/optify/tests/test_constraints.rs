use optify::schema::conditions::{Condition, ConditionExpression, Predicate, RegexWrapper};
use regex::Regex;
use serde_json::json;

#[test]
fn test_evaluate_equals_string() {
    let data = json!({
        "name": "John",
        "age": 30,
        "city": "New York"
    });

    // Test matching string
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/name".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("John"),
        },
    });
    assert!(condition.evaluate_with(&data));

    // Test non-matching string
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/name".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("Jane"),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_number() {
    let data = json!({
        "age": 30,
        "score": 95.5
    });

    // Test matching number
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/age".to_string(),
        operator_value: Predicate::Equals { equals: json!(30) },
    });
    assert!(condition.evaluate_with(&data));

    // Test matching float
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/score".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(95.5),
        },
    });
    assert!(condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_boolean() {
    let data = json!({
        "active": true,
        "disabled": false
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/active".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(true),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/disabled".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(false),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/active".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(false),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_null() {
    let data = json!({
        "value": null,
        "name": "test"
    });

    // Test matching null
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/value".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(null),
        },
    });
    assert!(condition.evaluate_with(&data));

    // Test non-null value against null
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/name".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(null),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_array() {
    let data = json!({
        "tags": ["rust", "programming", "systems"],
        "numbers": [1, 2, 3]
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/tags".to_string(),
        operator_value: Predicate::Equals {
            equals: json!(["rust", "programming", "systems"]),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers".to_string(),
        operator_value: Predicate::Equals {
            equals: json!([1, 2, 3]),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers".to_string(),
        operator_value: Predicate::Equals {
            equals: json!([1, 2, 3, 4]),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_object() {
    let data = json!({
        "user": {
            "name": "John",
            "age": 30,
            "city": "New York",
            "numbers": [1, 2, 3]
        }
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/user".to_string(),
        operator_value: Predicate::Equals {
            equals: json!({
                "name": "John",
                "age": 30,
                "city": "New York",
                "numbers": [1, 2, 3]
            }),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Not {
        not: Box::new(condition),
    };
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_nested_path() {
    let data = json!({
        "user": {
            "profile": {
                "name": "Alice"
            }
        }
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/user/profile/name".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("Alice"),
        },
    });
    assert!(condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_equals_missing_path() {
    let data = json!({
        "name": "John"
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/missing/path".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("value"),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_matches_basic() {
    let data = json!({
        "email": "john@example.com",
        "phone": "+1-555-1234"
    });

    // Test matching email pattern
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/email".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r".*@example\.com").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    // Test non-matching pattern
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/email".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r".*@gmail\.com").unwrap()),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_matches_regex_patterns() {
    let data = json!({
        "version": "v1.2.3",
        "id": "ABC123"
    });

    // Test version pattern
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/version".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"^v\d+\.\d+\.\d+$").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    // Test alphanumeric pattern
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/id".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"^[A-Z]+\d+$").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_matches_regex_patterns_with_array() {
    let data = json!({
        "numbers": [1, 2, 3],
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"1").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers/1".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"^2$").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers/1".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"^1$").unwrap()),
        },
    });
    assert!(!condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/numbers".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"4").unwrap()),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_matches_regex_patterns_with_boolean() {
    let data = json!({
        "active": true
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/active".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"^tr..$").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/active".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"false").unwrap()),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_matches_regex_patterns_with_object() {
    let data = json!({
        "user": {
            "age": 30,
            "city": "New York",
            "name": "John",
        }
    });

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/user".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(Regex::new(r"John").unwrap()),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/user".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(
                Regex::new(r#"^\{"age":30,"city":"New York","name":"John"\}$"#).unwrap(),
            ),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/user".to_string(),
        operator_value: Predicate::Matches {
            matches: RegexWrapper(
                Regex::new(r#"\{"blah":"John","age":30,"city":"New York"\}"#).unwrap(),
            ),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_and_group() {
    let data = json!({
        "name": "John",
        "age": "30",
        "city": "New York"
    });

    // Test all conditions true (should return true)
    let condition = ConditionExpression::And {
        and: vec![
            ConditionExpression::Condition(Condition {
                json_pointer: "/name".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("John"),
                },
            }),
            ConditionExpression::Condition(Condition {
                json_pointer: "/city".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("New York"),
                },
            }),
        ],
    };
    assert!(condition.evaluate_with(&data));

    // Test one condition false (should return false)
    let condition = ConditionExpression::And {
        and: vec![
            ConditionExpression::Condition(Condition {
                json_pointer: "/name".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("John"),
                },
            }),
            ConditionExpression::Condition(Condition {
                json_pointer: "/city".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("Los Angeles"),
                },
            }),
        ],
    };
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_or_group() {
    let data = json!({
        "status": "active",
        "type": "premium"
    });

    // Test at least one condition true (should return true)
    let condition = ConditionExpression::Or {
        or: vec![
            ConditionExpression::Condition(Condition {
                json_pointer: "/status".to_string(),
                operator_value: Predicate::Matches {
                    matches: RegexWrapper(Regex::new(r"^(active|new)$").unwrap()),
                },
            }),
            ConditionExpression::Condition(Condition {
                json_pointer: "/type".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("basic"),
                },
            }),
        ],
    };
    assert!(condition.evaluate_with(&data));

    // Test all conditions false (should return false)
    let condition = ConditionExpression::Or {
        or: vec![
            ConditionExpression::Condition(Condition {
                json_pointer: "/status".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("inactive"),
                },
            }),
            ConditionExpression::Condition(Condition {
                json_pointer: "/type".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("basic"),
                },
            }),
        ],
    };
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_not_group() {
    let data = json!({
        "enabled": "true"
    });

    // Test NOT with true condition (should return false)
    let condition = ConditionExpression::Not {
        not: Box::new(ConditionExpression::Condition(Condition {
            json_pointer: "/enabled".to_string(),
            operator_value: Predicate::Equals {
                equals: json!("true"),
            },
        })),
    };
    assert!(!condition.evaluate_with(&data));

    // Test NOT with false condition (should return true)
    let condition = ConditionExpression::Not {
        not: Box::new(ConditionExpression::Condition(Condition {
            json_pointer: "/enabled".to_string(),
            operator_value: Predicate::Equals {
                equals: json!("false"),
            },
        })),
    };
    assert!(condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_nested_groups() {
    let data = json!({
        "user": {
            "role": "admin",
            "active": "true"
        },
        "department": "IT"
    });

    // Test: (role = admin AND active = true) OR department = HR
    let condition = ConditionExpression::Or {
        or: vec![
            ConditionExpression::And {
                and: vec![
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/user/role".to_string(),
                        operator_value: Predicate::Equals {
                            equals: json!("admin"),
                        },
                    }),
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/user/active".to_string(),
                        operator_value: Predicate::Equals {
                            equals: json!("true"),
                        },
                    }),
                ],
            },
            ConditionExpression::Condition(Condition {
                json_pointer: "/department".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("HR"),
                },
            }),
        ],
    };
    assert!(condition.evaluate_with(&data)); // First AND group is true

    // Test: NOT((role = admin OR department = IT))
    let condition = ConditionExpression::Not {
        not: Box::new(ConditionExpression::Or {
            or: vec![
                ConditionExpression::Condition(Condition {
                    json_pointer: "/user/role".to_string(),
                    operator_value: Predicate::Equals {
                        equals: json!("admin"),
                    },
                }),
                ConditionExpression::Condition(Condition {
                    json_pointer: "/department".to_string(),
                    operator_value: Predicate::Equals {
                        equals: json!("IT"),
                    },
                }),
            ],
        }),
    };
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_empty_groups() {
    let data = json!({});

    // Empty AND should return true (all of nothing is true)
    let condition = ConditionExpression::And { and: vec![] };
    assert!(condition.evaluate_with(&data));

    // Empty OR should return false (none of nothing is true)
    let condition = ConditionExpression::Or { or: vec![] };
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_array_elements() {
    let data = json!({
        "tags": ["rust", "programming", "systems"],
        "numbers": [1, 2, 3]
    });

    // Test accessing array element
    let condition = ConditionExpression::And {
        and: vec![
            ConditionExpression::Condition(Condition {
                json_pointer: "/tags/0".to_string(),
                operator_value: Predicate::Equals {
                    equals: json!("rust"),
                },
            }),
            ConditionExpression::Condition(Condition {
                json_pointer: "/numbers/1".to_string(),
                operator_value: Predicate::Equals { equals: json!(2) },
            }),
        ],
    };
    assert!(condition.evaluate_with(&data));

    // Test accessing out of bounds array element
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/tags/10".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("rust"),
        },
    });
    assert!(!condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_special_characters_in_path() {
    let data = json!({
        "key/with/slashes": "value1",
        "key~with~tildes": "value2",
        "key.with.dots": "value3"
    });

    // JSON Pointer escapes: / -> ~1, ~ -> ~0
    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/key~1with~1slashes".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("value1"),
        },
    });
    assert!(condition.evaluate_with(&data));

    let condition = ConditionExpression::Condition(Condition {
        json_pointer: "/key~0with~0tildes".to_string(),
        operator_value: Predicate::Equals {
            equals: json!("value2"),
        },
    });
    assert!(condition.evaluate_with(&data));
}

#[test]
fn test_evaluate_complex_nested_conditions() {
    let data = json!({
        "request": {
            "method": "POST",
            "path": "/api/users",
            "headers": {
                "content-type": "application/json",
                "authorization": "Bearer token123"
            }
        },
        "timestamp": "2023-01-01T00:00:00Z"
    });

    // Complex condition: (method = POST AND path matches /api/.*) AND (content-type = json OR authorization matches Bearer.*)
    let condition = ConditionExpression::And {
        and: vec![
            ConditionExpression::And {
                and: vec![
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/request/method".to_string(),
                        operator_value: Predicate::Equals {
                            equals: json!("POST"),
                        },
                    }),
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/request/path".to_string(),
                        operator_value: Predicate::Matches {
                            matches: RegexWrapper(Regex::new(r"^/api/.*").unwrap()),
                        },
                    }),
                ],
            },
            ConditionExpression::Or {
                or: vec![
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/request/headers/content-type".to_string(),
                        operator_value: Predicate::Equals {
                            equals: json!("application/json"),
                        },
                    }),
                    ConditionExpression::Condition(Condition {
                        json_pointer: "/request/headers/authorization".to_string(),
                        operator_value: Predicate::Matches {
                            matches: RegexWrapper(Regex::new(r"^Bearer .*").unwrap()),
                        },
                    }),
                ],
            },
        ],
    };
    assert!(condition.evaluate_with(&data));
}
