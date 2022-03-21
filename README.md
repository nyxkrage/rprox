<div id="top"></div>
<!-- PROJECT LOGO -->
<br />
<div align="center">
<h3 align="center">rProx</h3>

  <p align="center">
    A very basic reverse proxy in Rust
    <br />
    <a href="https://docs.rs/rprox"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/nyxkrage/rprox/issues">Report Bug</a>
    ·
    <a href="https://github.com/nyxkrage/rprox/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

This is a basic reverse proxy, with hotreloading, and the abiltiy to write middleware in Golang\*

\*Not yet implemented

<p align="right">(<a href="#top">back to top</a>)</p>



### Built With

* [Rust](https://github.com/rust-lang/rust/)
* [Hyper](https://github.com/hyprium/hyper/)
* [Tokio](https://github.com/tokio-rs/tokio/)
* [Goscript](https://github.com/oxfeeefeee/goscript/)

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

* Rust - Install with [rustup](https://rustup.rs/)

### Installation

1. Clone the repo
   ```console
   $ git clone https://github.com/nyxkrage/rprox.git
   ```
1. Build with Cargo
   ```console
   $ cargo build
   ```
1. Run the project
   ```console
   $ cargo run
   ```

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

See [test.yaml](./test.yaml) for an example configuration

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [x] HTTP proxying  
- [ ] HTTPS with Rustls
- [ ] Middleware with Goscript
- [ ] CLI Argument parsing
- [x] hotreloading of config file

See the [open issues](https://github.com/nyxkrage/rprox/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
1. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
1. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
1. Push to the Branch (`git push origin feature/AmazingFeature`)
1. Open a Pull Request

<p align="right">(<a href="#top">back to top</a>)</p>


<!-- LICENSE -->
## License

Distributed under the MIT License. See [LICENSE](./LICENSE) for more information.

<p align="right">(<a href="#top">back to top</a>)</p>
