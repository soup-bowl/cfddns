# Cloudflare Dynamic DNS (CDDNS)

[![](https://img.shields.io/docker/pulls/soupbowl/cf-dynamicdns?logo=docker&logoColor=white)](https://hub.docker.com/r/soupbowl/cf-dynamicdns)
[![CodeFactor](https://www.codefactor.io/repository/github/soup-bowl/cloudflare-dynamicdns/badge)](https://www.codefactor.io/repository/github/soup-bowl/cloudflare-dynamicdns)
[![Build Container](https://github.com/soup-bowl/cloudflare-dynamicdns/actions/workflows/build.yml/badge.svg)](https://github.com/soup-bowl/cloudflare-dynamicdns/actions/workflows/build.yml)

With a specified Cloudflare DNS API token and a subdomain, this tool will detect your current IP address (using
[ident.me](https://api.ident.me/)) and sets your Cloudflare DNS record to that value. Optionally supports IPv6.

> [!WARNING]  
> This is not production ready, and undergoing pre-alpha changes - PRs will not be accepted at this time.

## Usage

### Docker

They can also be found on [Dockerhub](https://hub.docker.com/r/soupbowl/cf-dynamicdns).

See the help documentation for more details:

```bash
docker run ghcr.io/soup-bowl/cf-dynamicdns:latest --help
```

Run with **arguments**:

```bash
docker run ghcr.io/soup-bowl/cf-dynamicdns:latest \
  && --domain <your Dynamic DNS domain> \
  && --token <Your CF API Token>
```

Run with **environments**:

```bash
docker run ghcr.io/soup-bowl/cf-dynamicdns:latest \
  && --env CF_DOMAIN=<your Dynamic DNS domain> \
  && --env CF_TOKEN=<Your CF API Token>
```

Run via **Docker/Podman Compose**:

```yml
services:
  cfdydns:
    image: ghcr.io/soup-bowl/cf-dynamicdns:latest
    environment:
      CF_TOKEN: <token>
      CF_DOMAIN: example.com
```

### Executable

There are executables on the [Releases page](https://github.com/soup-bowl/cloudflare-dynamicdns/releases/latest). These are compiled as single-binary executables for multiple architectures (AMD64, ARM64).

Below is a one-liner script to download and install to the binary path on Linux (requires sudo).

```bash
wget -O /tmp/cddns.zip "https://github.com/soup-bowl/cloudflare-dynamicdns/releases/download/0.2/cddns-0.2-linux-$(dpkg --print-architecture).zip" \
  && unzip /tmp/cddns.zip -d /tmp \
  && rm /tmp/cddns.zip \
  && chmod +x /tmp/cddns \
  && sudo chown root:root /tmp/cddns \
  && sudo mv /tmp/cddns /bin/
```

Verify by running `cddns --version`.

### Native

This is a Rust application that compiles to a single static binary with minimal dependencies.

To build from source:

```bash
cargo build --release
```

The binary will be located at `target/release/cddns`.

To run:

```bash
./target/release/cddns --domain <your Dynamic DNS domain> --token <Your CF API Token>
```

(Arguments can be omitted if you have the values in your environment).

## Getting your Cloudflare Token

Visit the [API Tokens segment of your Cloudflare Profile](https://dash.cloudflare.com/profile/api-tokens). Create an
**API Token** (not an **API Key**), and select to use the Edit Zone DNS template.

How you fill the rest is up to you, but I recommend specifying the **Zone Resource** to **Include**, **Specific zone**,
and specify the domain where your Dynamic DNS will be.

After review, the system will output an **API Token**. This is what the tool wants as either `--token` or `CF_TOKEN`
argument. The `--domain`/`CF_DOMAIN` argument **must match** whatever zone you specified for the token, or at least be
applicable within the scope you set.
