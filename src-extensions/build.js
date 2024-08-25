const { inject } = require("postject");
const fs = require("node:fs");
const which = require("which");
const { exec } = require("node:child_process");

const nodePath = which
  .sync("node", {
    all: true,
  })
  .filter(
    (v) => !v.startsWith("/tmp") || !v.includes("Temp") || !v.includes("/T/"),
  )[0];
console.log(nodePath);

fs.copyFile(nodePath, "./server", (err) => {
  if (err) console.error(err);

  const buffer = fs.readFileSync("./sea-prep.blob");
  inject("server", "NODE_SEA_BLOB", buffer, {
    sentinelFuse: "NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2",
  }).then(() => {
    console.log("done injectng");

    const proc = exec(
      "rustc -Vv | grep host | cut -f2 -d' '",
      (err, stdout, stderr) => {
        console.log("inside exec", err, stdout);
        if (err) {
          console.error("error", err);
          return;
        }

        fs.mkdirSync("../src-tauri/binaries", {
          recursive: true,
        });

        fs.renameSync(
          "./server",
          `../src-tauri/binaries/exthost-${stdout.trim()}`,
        );
        fs.copyFileSync(
          "./dist/bridge.js",
          `../src-tauri/binaries/bridge.js-${stdout.trim()}`,
        );
        fs.copyFileSync(
          "./dist/events.js",
          `../src-tauri/binaries/events.js-${stdout.trim()}`,
        );
        fs.copyFileSync(
          "./dist/setup-sandbox.js",
          `../src-tauri/binaries/setup-sandbox.js-${stdout.trim()}`,
        );
        fs.copyFileSync(
          "./dist/setup-node-sandbox.js",
          `../src-tauri/binaries/setup-node-sandbox.js-${stdout.trim()}`,
        );
      },
    );

    console.log("finished execution");
  });
});
