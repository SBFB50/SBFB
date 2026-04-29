// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fairness observability metrics — pure math, no DB access
//! (Sprint 41 Phase A, port of fairness.py S23).

use std::collections::HashSet;

pub fn compute_gini(contributions: &[f64]) -> f64 {
    let n = contributions.len();
    if n <= 1 {
        return 0.0;
    }
    let total: f64 = contributions.iter().sum();
    if total == 0.0 {
        return 0.0;
    }
    let mut sorted = contributions.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut weighted_sum = 0.0;
    for (i, c) in sorted.iter().enumerate() {
        weighted_sum += (2.0 * (i as f64 + 1.0) - n as f64 - 1.0) * c;
    }
    weighted_sum / (n as f64 * total)
}

pub fn compute_top_k_share(contributions: &[f64], k_pct: usize) -> f64 {
    if contributions.is_empty() {
        return 0.0;
    }
    let total: f64 = contributions.iter().sum();
    if total == 0.0 {
        return 0.0;
    }
    let count = (contributions.len() * k_pct / 100).max(1);
    let mut sorted = contributions.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let top_sum: f64 = sorted.iter().take(count).sum();
    top_sum / total
}

pub fn compute_churn_rate(previous: &HashSet<String>, current: &HashSet<String>) -> f64 {
    if previous.is_empty() {
        return 0.0;
    }
    let departed = previous.difference(current).count();
    departed as f64 / previous.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gini_uniform_distribution() {
        assert!((compute_gini(&[10.0, 10.0, 10.0, 10.0]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gini_complete_inequality() {
        let result = compute_gini(&[0.0, 0.0, 0.0, 100.0]);
        assert!(result > 0.7);
    }

    #[test]
    fn gini_empty_and_single() {
        assert!((compute_gini(&[]) - 0.0).abs() < f64::EPSILON);
        assert!((compute_gini(&[42.0]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn top_k_share_all_equal() {
        let values = vec![10.0; 100];
        let share = compute_top_k_share(&values, 10);
        assert!((share - 0.1).abs() < 0.02);
    }

    #[test]
    fn top_k_share_empty() {
        assert!((compute_top_k_share(&[], 5) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn churn_rate_no_change() {
        let prev: HashSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let curr = prev.clone();
        assert!((compute_churn_rate(&prev, &curr) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn churn_rate_total_change() {
        let prev: HashSet<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let curr: HashSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
        assert!((compute_churn_rate(&prev, &curr) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn churn_rate_empty_previous() {
        let prev: HashSet<String> = HashSet::new();
        let curr: HashSet<String> = ["a"].iter().map(|s| s.to_string()).collect();
        assert!((compute_churn_rate(&prev, &curr) - 0.0).abs() < f64::EPSILON);
    }
}
