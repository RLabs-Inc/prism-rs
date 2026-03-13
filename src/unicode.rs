use unicode_segmentation::UnicodeSegmentation;

/// A grapheme cluster segment with byte positions
pub struct GraphemeSegment {
    pub segment: String,
    pub start: usize,
    pub end: usize,
}

/// Split text into grapheme cluster segments with byte offsets
pub fn grapheme_segments(text: &str) -> Vec<GraphemeSegment> {
    if text.is_empty() {
        return Vec::new();
    }
    text.grapheme_indices(true)
        .map(|(start, seg)| GraphemeSegment {
            segment: seg.to_string(),
            start,
            end: start + seg.len(),
        })
        .collect()
}

/// Find the previous grapheme boundary before index (byte offset)
pub fn previous_grapheme_boundary(text: &str, index: usize) -> usize {
    if index == 0 || text.is_empty() {
        return 0;
    }
    let clamped = index.min(text.len());
    for seg in grapheme_segments(text) {
        if clamped <= seg.start {
            return seg.start;
        }
        if clamped <= seg.end {
            return seg.start;
        }
    }
    text.len()
}

/// Find the next grapheme boundary after index (byte offset)
pub fn next_grapheme_boundary(text: &str, index: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    let clamped = index.min(text.len());
    for seg in grapheme_segments(text) {
        if clamped < seg.start {
            return seg.start;
        }
        if clamped < seg.end {
            return seg.end;
        }
    }
    text.len()
}

/// Snap an index to the nearest grapheme boundary (forward)
pub fn normalize_grapheme_boundary(text: &str, index: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    let clamped = index.min(text.len());
    if clamped == 0 || clamped == text.len() {
        return clamped;
    }
    for seg in grapheme_segments(text) {
        if clamped == seg.start || clamped == seg.end {
            return clamped;
        }
        if clamped > seg.start && clamped < seg.end {
            return seg.end;
        }
    }
    clamped
}
