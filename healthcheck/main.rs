use std::{env, process::ExitCode};

// defaults to the 8080 port
// takes an APP_PORT var but APP_PORT is not implemented in the app 
// I didn't do that since this is not really the point of the PR
// this just proves it can be done and can be built on
fn main() -> ExitCode {
    let port = env::var("APP_PORT").unwrap_or_else(|_| String::from("8080"));

    // uses http, if you eventually add TLS support then you need more dependencies
    // as mentioned in this blog you would need 17 more dependencies, to have rustls
    // making it a 	531kb -> 1.2mb binary
    // which at that point it approaches the size of wget from busybox
    // so that could be more efficient at that point
    // https://natalia.dev/blog/2023/03/docker-health-checks-on-distroless-containers-with-rust
    let endpoint = format!("http://localhost:{}/", port);

    let res = minreq::get(endpoint).send();

    if res.is_err() {
        println!("{}", res.unwrap_err());
        return ExitCode::from(1);
    }

    let code = res.unwrap().status_code;

    // I altered the .rs file in the blog above
    // this is more strict, it will only claim healthy if code 200-399
    if (200..=399).contains(&code) {
        return ExitCode::from(0)
    }

    println!("Did not recieve a 200-399 code, the service is likely down {}", code);
    return ExitCode::from(1);
}
