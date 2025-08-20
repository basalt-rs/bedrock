use bedrock::templates::*;

#[tokio::main]
async fn main() {
    let template: Template = include_str!("./reverse-template.py")
        .to_string()
        .try_into()
        .unwrap();

    let mut x = template.populated(indoc::indoc!(
        r#"
            def reverse(line: str) -> str:
                out = ''
                for c in reversed(line):
                    out += c
                return out
            "#
    ));

    use tokio::io;

    eprintln!("Populated:");
    io::copy(&mut x, &mut io::stdout()).await.unwrap();

    // eprintln!("Solution:");
    // let mut solution = template.solution().unwrap();
    // io::copy(&mut solution, &mut io::stdout()).await.unwrap();
}
