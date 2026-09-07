use secret_sorbet_pw::{LoginInput, generate_login_password};

fn main() {
    let input = LoginInput::new("long private passphrase", "github.com", "ted@example.com");

    println!("{}", generate_login_password(&input).unwrap());
}
