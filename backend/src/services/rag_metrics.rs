//! # RAG Quality Metrics
//!
//! Lightweight, in-process quality scoring for every email generation.
//! No external API calls — all computation is done in Rust after generation completes.
//!
//! ## Metrics
//!
//! ### Faithfulness Score (0.0 – 1.0)
//! Measures how well the generated reply stays grounded in the retrieved context chunks.
//! **Method:** Token-overlap (Jaccard) between each reply sentence and the context corpus.
//! A sentence with Jaccard ≥ 0.15 against any context chunk is counted as "grounded".
//! `faithfulness = grounded_sentences / total_sentences`
//!
//! High score → reply closely follows retrieved context (low hallucination risk).
//! Low score  → reply likely introduces facts not in context.
//!
//! ### Context Precision (0.0 – 1.0)
//! Average hybrid similarity score of the retrieved chunks (already computed during retrieval).
//! Represents how relevant the knowledge base chunks were to the query.
//! High score → retrieval was highly relevant; low score → retrieval was imprecise.

/// Compute how grounded a generated reply is in its retrieved context chunks.
///
/// Algorithm:
/// 1. Split reply into sentences (naive: split on `.`, `!`, `?` + newlines).
/// 2. Tokenize each sentence and each chunk into lowercase word sets.
/// 3. For each sentence, compute Jaccard similarity against every chunk.
/// 4. A sentence is "grounded" if max Jaccard ≥ `JACCARD_THRESHOLD`.
/// 5. Return fraction of grounded sentences.
pub fn compute_faithfulness(reply: &str, context_chunks: &[&str]) -> f32 {
    const JACCARD_THRESHOLD: f32 = 0.12;
    const MIN_TOKENS: usize = 4; // ignore very short sentences

    if context_chunks.is_empty() || reply.is_empty() {
        return 0.0;
    }

    // Pre-compute token sets for all context chunks
    let chunk_token_sets: Vec<std::collections::HashSet<&str>> = context_chunks
        .iter()
        .map(|chunk| tokenize(chunk))
        .collect();

    // Split reply into sentences
    let sentences: Vec<&str> = reply
        .split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        return 0.0;
    }

    let mut grounded = 0usize;

    for sentence in &sentences {
        let s_tokens = tokenize(sentence);
        if s_tokens.len() < MIN_TOKENS {
            // Short sentences are too ambiguous — count as grounded to avoid penalising
            grounded += 1;
            continue;
        }

        // Check against every chunk; a single match is enough
        let is_grounded = chunk_token_sets.iter().any(|chunk_tokens| {
            jaccard(&s_tokens, chunk_tokens) >= JACCARD_THRESHOLD
        });

        if is_grounded {
            grounded += 1;
        }
    }

    grounded as f32 / sentences.len() as f32
}

/// Compute the average hybrid score of retrieved contexts as a precision measure.
///
/// This is the mean of the `score` field already computed in `search_all_context`.
/// Range: 0.0 – 1.0. Higher = retrieved chunks were more relevant.
pub fn compute_context_precision(scores: &[f64]) -> f32 {
    if scores.is_empty() {
        return 0.0;
    }
    let sum: f64 = scores.iter().sum();
    (sum / scores.len() as f64) as f32
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn tokenize(text: &str) -> std::collections::HashSet<&str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .collect()
}

fn jaccard(a: &std::collections::HashSet<&str>, b: &std::collections::HashSet<&str>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = (a.len() + b.len()) as f32 - intersection;
    if union == 0.0 { 0.0 } else { intersection / union }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faithfulness_empty() {
        assert_eq!(compute_faithfulness("", &[]), 0.0);
        assert_eq!(compute_faithfulness("hello world", &[]), 0.0);
    }

    #[test]
    fn test_faithfulness_perfect_match() {
        let context = ["The deadline for the project is Friday June 6th at 5pm."];
        let reply = "The deadline for the project is Friday June 6th at 5pm. Please confirm.";
        let score = compute_faithfulness(reply, &context);
        assert!(score > 0.5, "Expected faithfulness > 0.5, got {}", score);
    }

    #[test]
    fn test_faithfulness_no_match() {
        let context = ["Our refund policy allows 30 days for returns."];
        // Reply is completely unrelated — but short sentences get credit, so just check low
        let reply = "The weather in Porto is sunny and warm today, with temperatures reaching 28 degrees Celsius.";
        let score = compute_faithfulness(reply, &context);
        assert!(score < 0.5, "Expected faithfulness < 0.5, got {}", score);
    }

    #[test]
    fn test_context_precision_empty() {
        assert_eq!(compute_context_precision(&[]), 0.0);
    }

    #[test]
    fn test_context_precision_average() {
        let scores = [0.8, 0.6, 0.7];
        let precision = compute_context_precision(&scores);
        assert!((precision - 0.7f32).abs() < 0.01, "Expected ~0.7, got {}", precision);
    }
}
