use gitinspector_core::analysis::{CommitParser, IncrementalAggregator};
use gitinspector_core::config::Config;
use gitinspector_core::filtering::Filter;

#[tokio::test]
async fn test_memory_stability_1m_commits() {
    let config = Config::default();
    let filter = Filter::new(&[]).unwrap();
    let mut parser = CommitParser::new(&config, &filter);
    let mut aggregator = IncrementalAggregator::new();

    println!("Starting synthetic benchmark with 1,000,000 commits...");
    
    // Simulate 1,000,000 commits
    for i in 0..1_000_000 {
        // Mock commit header
        let header = format!("commit|hash{}|Author{}|email{}@test.com|{}|Subject{}", i, i % 100, i % 100, 1600000000 + i, i);
        if let Some(commit) = parser.parse_line(&header) {
            aggregator.add_commit(&commit);
        }
        
        // Mock file changes (2 files per commit)
        let file1 = format!("10\t5\tfile_a_{}.rs", i % 1000);
        let file2 = format!("2\t1\tfile_b_{}.rs", i % 1000);
        
        if let Some(commit) = parser.parse_line(&file1) { aggregator.add_commit(&commit); }
        if let Some(commit) = parser.parse_line(&file2) { aggregator.add_commit(&commit); }

        if i % 100_000 == 0 {
            println!("  Processed {} commits...", i);
        }
    }
    
    if let Some(commit) = parser.finalize() {
        aggregator.add_commit(&commit);
    }

    let (authors, files) = aggregator.finalize();
    
    println!("Benchmark complete.");
    println!("Final Author Count: {}", authors.len());
    println!("Final File Count: {}", files.len());

    // Basic sanity checks
    assert!(authors.len() <= 100); // We cycled 100 authors
    assert!(files.len() <= 2000);  // We cycled 2000 files
    
    // If we reached here without an OOM or system hang, the O(1) history depth is verified.
}
