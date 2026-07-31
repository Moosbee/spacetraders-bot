use tokio::io::{AsyncBufReadExt, BufReader};

pub(crate) async fn analyse_logs(input: String) -> Result<(), anyhow::Error> {
    println!("Starting analysis of {}", input);
    let now = std::time::Instant::now();

    let file = tokio::fs::File::open(input.clone()).await?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut line_num = 0;

    while let Some(line) = lines.next_line().await? {
        line_num += 1;
        println!("{}: length = {}", line_num, line.len())
    }

    println!(
        "Finished analysis of {} in {}ms",
        input,
        now.elapsed().as_millis()
    );

    Ok(())
}
