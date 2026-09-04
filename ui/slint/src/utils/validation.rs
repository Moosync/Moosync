use regex::Regex;
use tracing::warn;

#[tracing::instrument(level = "debug", skip_all)]
pub fn validate_input(value: &str, rule: &str) -> bool {
    let trimmed_rule = rule.trim();
    if trimmed_rule.is_empty() {
        return true;
    }

    if is_arithmetic_expression(trimmed_rule) {
        return validate_arithmetic(value, trimmed_rule);
    }

    validate_regex(value, trimmed_rule)
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_arithmetic_expression(rule: &str) -> bool {
    let has_operator = rule.contains("==")
        || rule.contains("!=")
        || rule.contains(">=")
        || rule.contains("<=")
        || rule.starts_with('>')
        || rule.starts_with('<')
        || rule.contains("..")
        || rule.contains("&&")
        || rule.contains("||");

    let is_bracket_regex = rule.starts_with('^')
        || rule.starts_with('[')
        || rule.starts_with('(')
        || rule.ends_with('$');

    if is_bracket_regex {
        return false;
    }

    has_operator
}

#[tracing::instrument(level = "debug", skip_all)]
fn validate_arithmetic(value: &str, rule: &str) -> bool {
    let Ok(number) = value.trim().parse::<f64>() else {
        return false;
    };

    let or_branches: Vec<&str> = rule.split("||").collect();
    for branch in or_branches {
        if evaluate_and_branch(number, branch.trim()) {
            return true;
        }
    }

    false
}

#[tracing::instrument(level = "debug", skip_all)]
fn evaluate_and_branch(number: f64, branch: &str) -> bool {
    let and_clauses = branch.split("&&");
    for clause in and_clauses {
        if !evaluate_clause(number, clause.trim()) {
            return false;
        }
    }

    true
}

#[tracing::instrument(level = "debug", skip_all)]
fn evaluate_clause(number: f64, clause: &str) -> bool {
    let trimmed = clause.trim();
    if trimmed.is_empty() {
        return true;
    }

    if trimmed.contains("..") {
        return evaluate_range(number, trimmed);
    }

    if let Some(rest) = trimmed.strip_prefix(">=") {
        return evaluate_comparison(number, rest, |n, target| n >= target);
    }

    if let Some(rest) = trimmed.strip_prefix("<=") {
        return evaluate_comparison(number, rest, |n, target| n <= target);
    }

    if let Some(rest) = trimmed.strip_prefix('>') {
        return evaluate_comparison(number, rest, |n, target| n > target);
    }

    if let Some(rest) = trimmed.strip_prefix('<') {
        return evaluate_comparison(number, rest, |n, target| n < target);
    }

    if let Some(rest) = trimmed.strip_prefix("==") {
        return evaluate_comparison(number, rest, |n, target| (n - target).abs() < f64::EPSILON);
    }

    if let Some(rest) = trimmed.strip_prefix("!=") {
        return evaluate_comparison(number, rest, |n, target| (n - target).abs() >= f64::EPSILON);
    }

    if let Ok(target) = trimmed.parse::<f64>() {
        return (number - target).abs() < f64::EPSILON;
    }

    false
}

#[tracing::instrument(level = "debug", skip_all)]
fn evaluate_comparison<F>(number: f64, rest: &str, compare: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    let Ok(target) = rest.trim().parse::<f64>() else {
        return false;
    };

    compare(number, target)
}

#[tracing::instrument(level = "debug", skip_all)]
fn evaluate_range(number: f64, range_str: &str) -> bool
where
{
    let is_inclusive = range_str.contains("..=");
    let parts: Vec<&str> = if is_inclusive {
        range_str.split("..=").collect()
    } else {
        range_str.split("..").collect()
    };

    if parts.len() != 2 {
        return false;
    }

    let Ok(min) = parts[0].trim().parse::<f64>() else {
        return false;
    };
    let Ok(max) = parts[1].trim().parse::<f64>() else {
        return false;
    };

    if is_inclusive {
        return number >= min && number <= max;
    }

    number >= min && number < max
}

#[tracing::instrument(level = "debug", skip_all)]
fn validate_regex(value: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(compiled_regex) => compiled_regex.is_match(value),
        Err(err) => {
            warn!("Invalid regex '{}': {}", pattern, err);
            false
        }
    }
}
