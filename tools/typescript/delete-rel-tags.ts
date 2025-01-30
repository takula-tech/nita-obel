#!/usr/bin/env -S deno run --allow-all
import $ from "@david/dax";
await $`git tag -l | xargs git push --delete origin && git tag -l | xargs git tag -d`;
console.log("All release tags deleted");
