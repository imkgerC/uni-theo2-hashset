use gnuplot::{AxesCommon, Caption, Figure, Graph};
use std::fs::OpenOptions;
use std::io::Write;

pub fn print_header(name: &str, load_factors: &[f64]) {
    let mut out = format!("{:20}", name);
    for (i, lambda) in load_factors.iter().enumerate() {
        let lambda = format!("{:.0}%", lambda * 100_f64);
        out.push_str(&format!("{:^5}", lambda));
        if i != load_factors.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
}

pub fn print_subtable(name: &str, stats: &[(f32, f64, f32, f64)], load_factors: &[f64]) {
    println!();
    print_header(name, load_factors);
    let mut out = format!("{:20}", "+ collisions");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].0));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "+ time[ns]");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].1));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "- collisions");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].2));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "- time[ns]");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].3));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
}

pub fn write_csv(all_stats: &[(String, Vec<(f32, f64, f32, f64)>)], load_factors: &[f64]) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("hashset_data.csv")
        .expect("Could not open file to write output analysis to");
    let mut header = String::new();
    header.push_str("\"Name\",");
    for lambda in load_factors {
        let percentage = format!("{:.0}%", lambda * 100_f64);
        header.push_str(&format!("\"Success Collisions({0})\",\"Success Time({0})[ns]\",\"Failures Collisions({0})\",\"Failures Time({0})[ns]\",", percentage));
    }
    header.push_str("\r\n");
    file.write_all(header.as_bytes())
        .expect("Could not write to file");
    for (name, stats) in all_stats {
        let mut f = format!("\"{}\"", name);
        for stat in stats {
            f.push_str(&format!(",{},{},{},{}", stat.0, stat.1, stat.2, stat.3));
        }
        f.push_str("\n");
        file.write_all(f.as_bytes())
            .expect("Could not write to file");
    }
}

pub fn write_graphs(all_stats: &[(String, Vec<(f32, f64, f32, f64)>)], load_factors: &[f64], element_count: usize) {
    let mut fg = Figure::new();
    let ax = fg
        .axes2d()
        .set_title("Collisions on success", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("Number of elements", &[])
        .set_y_label("Collisions", &[]);
    for (name, stats) in all_stats {
        ax.lines(
            load_factors
                .iter()
                .map(|x| (x * element_count as f64) as usize),
            stats.iter().map(|x| x.0),
            &[Caption(&name)],
        );
    }
    fg.save_to_png("./graphs/successful_collisions.png", 1920, 1080)
        .expect("Could not save file");

    let mut fg = Figure::new();
    let ax = fg
        .axes2d()
        .set_title("Collisions on failure", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("Number of elements", &[])
        .set_y_label("Collisions", &[]);
    for (name, stats) in all_stats {
        ax.lines(
            load_factors
                .iter()
                .map(|x| (x * element_count as f64) as usize),
            stats.iter().map(|x| x.2),
            &[Caption(&name)],
        );
    }
    fg.save_to_png("./graphs/failure_collisions.png", 1920, 1080)
        .expect("Could not save file");

    let mut fg = Figure::new();
    let ax = fg
        .axes2d()
        .set_title("Time on success", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("Number of elements", &[])
        .set_y_label("time[ns]", &[]);
    for (name, stats) in all_stats {
        ax.lines(
            load_factors
                .iter()
                .map(|x| (x * element_count as f64) as usize),
            stats.iter().map(|x| x.1),
            &[Caption(&name)],
        );
    }
    fg.save_to_png("./graphs/successful_time.png", 1920, 1080)
        .expect("Could not save file");

    let mut fg = Figure::new();
    let ax = fg
        .axes2d()
        .set_title("Time on failure", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("Number of elements", &[])
        .set_y_label("time[ns]", &[]);
    for (name, stats) in all_stats {
        ax.lines(
            load_factors
                .iter()
                .map(|x| (x * element_count as f64) as usize),
            stats.iter().map(|x| x.3),
            &[Caption(name)],
        );
    }
    fg.save_to_png("./graphs/failure_time.png", 1920, 1080)
        .expect("Could not save file");
}