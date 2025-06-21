const { execa } = require("execa");
const fs = require("fs");

let extension = "";
let fileName = "server";
if (process.platform === "win32") {
	extension = ".exe";
}

async function main() {
	const rustInfo = (await execa("rustc", ["-vV"])).stdout;
	const targetTriple = /host: (\S+)/g.exec(rustInfo)[1];
	if (!targetTriple) {
		console.error("Failed to determine platform target triple");
	}

    // Check if binary file has been renamed already
	if (fs.existsSync(`src-tauri/bin/${fileName}${extension}`)) {
		console.log(
			`Renaming ${fileName}${extension} to ${fileName}-${targetTriple}${extension}`
		);
		fs.renameSync(
			`src-tauri/bin/${fileName}${extension}`,
			`src-tauri/bin/${fileName}-${targetTriple}${extension}`
		);
	} else {
		console.log(
			`binary File (${fileName}${extension}) does not exist, skipping rename.`
		);
	}
}

main().catch((e) => {
	throw e;
});
