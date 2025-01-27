#!/usr/bin/env zx
import { $, fs, sleep } from "zx";
import { consola } from "consola";
import { env, exit } from "process";
import tomlJson from "toml-json";

//Makes all commands print logs to the console by default
$.verbose = true;
const dependencies = tomlJson({fileUrl: "Cargo.toml"});
await $`whoami`;
consola.info({dependencies});
