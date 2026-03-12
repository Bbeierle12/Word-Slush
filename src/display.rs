use comfy_table::{Table, ContentArrangement};
use crate::counter::WordCount;

pub fn render_table(counts: &[WordCount], limit: usize) {
    let total: u32 = counts.iter().map(|w| w.count).sum();

    if total == 0 {
        println!("No words found.");
        return;
    }

    let display_counts = if limit == 0 { counts } else { &counts[..limit.min(counts.len())] };

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Rank", "Word", "Count", "% of Total"]);

    for (i, wc) in display_counts.iter().enumerate() {
        let pct = (wc.count as f64 / total as f64) * 100.0;
        table.add_row(vec![
            format!("{}", i + 1),
            wc.word.clone(),
            format!("{}", wc.count),
            format!("{:.2}%", pct),
        ]);
    }

    println!("{table}");
    println!("\nTotal words (scoped): {total}");
}
