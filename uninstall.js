#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { exec } = require("child_process");

const cargoDir = path.dirname("$HOME" + ".cargo");

// check if directory exists
if (fs.existsSync(cargoDir)) {
  //   console.log("Cargo found.");
} else {
  const setCargo = 'PATH="/$HOME/.cargo/bin:${PATH}"';
  console.log("Installing deps [cargo].");

  exec(
    `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && ${setCargo}`,
    (error) => {
      if (error) {
        console.log(
          "curl failed! Curl may not be installed on the OS. View https://curl.se/download.html to install."
        );
        console.log(error);
      }
    }
  );
}

const binp = path.join(cargoDir, "bin", "website_crawler");

if (fs.existsSync(binp)) {
  console.log("Uninstalling website_crawler...");
  exec(`cargo uninstall a11ywatch_cli`, (error, stdout, stderr) => {
    console.log(stdout);
    if (error || stderr) {
      console.log(error || stderr);
    }
  });
} else {
  console.log("website_crawler not found skipping!");
}
