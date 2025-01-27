#!/usr/bin/env -S deno run --allow-all

import $ from "@david/dax";
import * as TOML from "@std/toml";

// Define root path constant using fs api
const rootPath = new URL("../..\n", import.meta.url).pathname;

// Read and parse Cargo.toml
const cargoToml = await Deno.readTextFile(`${rootPath}Cargo.toml`);
const cargoConfig = TOML.parse(cargoToml);

// Extract workspace dependencies starting with 'obel_'
const workspaceDeps = (cargoConfig.workspace as any)?.dependencies || {};
const depNames = Object.keys(workspaceDeps).filter((name) => name.startsWith("obel_"));

// Log dependency names
console.log("All obel_* crate names:");
depNames.forEach((name) => console.log(`- ${name}`));

// Fetch versions from crates.io using Promise.allSettled
console.log("\nFetching latest versions from crates.io...");

const fetchCrateVersion = async (name: string) => {
  const response = await fetch(`https://crates.io/api/v1/crates/${name}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch versions: ${response.statusText}`);
  }
  const data = await response.json();
  return {
    name,
    version: data.versions[0]?.num || "unknown",
    path: workspaceDeps[name].path,
  };
};

// Create an array of promises with delay between each request
const fetchPromises = depNames.map(
  (name, index) =>
    new Promise((resolve) =>
      setTimeout(
        () => resolve(fetchCrateVersion(name)),
        index * 500 // 500ms delay between requests
      )
    )
);

// Wait for all requests to complete
const results = await Promise.allSettled(fetchPromises);

// Store results in a Map
const crateVersions = new Map<string, { version: string; path: string }>();

// Process results and store in map
results.forEach((result, index) => {
  const name = depNames[index];
  if (result.status === "fulfilled") {
    const data = result.value as { name: string; version: string; path: string };
    crateVersions.set(data.name, { version: data.version, path: data.path });
    console.log(`${data.name}: ${data.version} (${data.path})`);
  } else {
    console.error(`\nError fetching version for ${name}:`, result.reason);
    crateVersions.set(name, { version: "unknown", path: workspaceDeps[name].path });
  }
});

console.log(`\n`);
await Promise.all(
  depNames.map(async (name) => {
    const crateInfo = crateVersions.get(name);
    if (crateInfo) {
      const version = `${name}@${crateInfo.version}`;
      const manifestPath = `${rootPath}${crateInfo.path}`;
      const cmd = `cd ${manifestPath} && release-plz set-version ${version}`;
      console.log(cmd);
      // await $`${cmd}`;
    }
  })
);
