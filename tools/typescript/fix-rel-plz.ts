#!/usr/bin/env -S deno run --allow-all

// to fix the issue related to release-plz with the following error message from github action output:
// INFO  obel: the local package has already a different version with respect to the registry package,
// so release-plz will not update it
// this could happen when the release pr pipeline fails for some reasons, leaving the broken crate version
// between local and remote crates.

import $ from "@david/dax";
import * as TOML from "@std/toml";
import process from "node:process";

// Define root path constant using fs api
const rootPath = new URL("../..\n", import.meta.url).pathname;

// Read and parse Cargo.toml
const cargoToml = await Deno.readTextFile(`${rootPath}Cargo.toml`);
const cargoConfig = TOML.parse(cargoToml);

const crates = new Map<string, { name: string; path: string; remoteVersion: string; localVersion: string }>();

// Extract workspace dependencies starting with 'obel_'
// deno-lint-ignore no-explicit-any
const workspaceDeps = (cargoConfig.workspace as any)?.dependencies || {};
Object.keys(workspaceDeps)
  .filter((name) => name.startsWith("obel_"))
  .forEach((v) => crates.set(v, { localVersion: workspaceDeps[v].version, path: workspaceDeps[v].path, name: v, remoteVersion: "" }));

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
const fetchPromises = Array.from(crates.keys()).map(
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

// Process results and store in map
results.forEach((result, index) => {
  if (result.status === "fulfilled") {
    const data = result.value as { name: string; version: string };
    const remote = crates.get(data.name);
    if (!remote) return;
    crates.set(data.name, { ...remote, remoteVersion: data.version });
    console.log(data);
  } else {
    console.error(`\nError fetching version for ${name}:`, result.reason);
  }
});

console.log(`\n`);
let pushToMainBranch = false;
await Promise.all(
  Array.from(crates).map(async ([name, value]) => {
    if (value.localVersion === value.remoteVersion) return;
    pushToMainBranch = true;
    const version = `${name}@${value.remoteVersion}`;
    const manifestPath = `${rootPath}${value.path}`;
    const relPlz = `release-plz set-version ${version}`;
    console.log(relPlz);
    await $`cd ${manifestPath} && ${relPlz}`;
  })
);

if (pushToMainBranch) {
  await $`cd ${rootPath}`;
  await $`git add --all`;
  await $`git commit -m "chore(skip): fix release-plz version"`;
}

console.log("Fix done. Please force push to main branch\ngit push origin [your-branch-name]:main -f");
