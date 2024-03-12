# tsc2046

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/EdBuilds/TSC2046/ci.yml?style=for-the-badge&labelColor=555555)
![Crates.io Version](https://img.shields.io/crates/v/tsc2046?style=for-the-badge&labelColor=555555)
![docs.rs](https://img.shields.io/docsrs/tsc2046?style=for-the-badge&labelColor=555555)


TSC2046 SPI 4-Wire Touch Screen Controller driver

## Overview

This crate is a platform-agnostic Rust driver for the TSC2046 4-wire touch screen controller. This crate provides a high-level interface to interact with the TSC2046 chip, allowing you to read the X and Y coordinates of a touch, as well as calculate the pressure applied on the touch screen.

The driver is designed to work with any hardware abstraction layer (HAL) that implements the embedded-hal v1.0.0 traits and works in `no_std` environments.

## Features
- Read X and Y coordinates of touch
- Configure the interrupt output of the chip
- Touch detection
- Touch pressure calculation

## Installation

To use this crate in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
tsc2046 = "0.1.0"