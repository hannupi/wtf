use std::process::Command;

fn main() {
    let output = Command::new("tmux")
        .arg("capture-pane")
        .arg("-p")
        .arg("-S")
        .arg("-50")
        .output()
        .expect("Failed to execute tmux capture");
    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = content.lines().collect();

    let (index, prompt) = lines
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, line)| line.trim_start().contains("$"))
        .nth(1) // first result is calling this script
        .expect("Failed to find prompt");

    let (info, cmd) = prompt
        .find("$")
        .map(|pos| {
            let info = prompt[..pos].split(":").next().unwrap().trim();
            let cmd = prompt[pos + 2..].trim();
            (info, cmd)
        })
        .expect("Failed to trim user info/cmd");

    let relevant_lines: Vec<&str> = lines[index..]
        .iter()
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with(info))
        .cloned()
        .collect();
    dbg!(relevant_lines);

    //let cmd_stdout: Vec<&str> = lines
    //    .iter()
    //    .skip_while(|line| !line.starts_with(info))
    //    .skip(1)
    //    .cloned()
    //    .collect();
    //dbg!(cmd_stdout);
}
