import { createConnection } from "node:net";
import { ExtensionHostIPCHandler } from "./sandbox";
import { EventEmitter } from "node:events";

const IPC_PATH =
	"/home/ovenoboyo/.local/share/app.moosync.moosync/extensions/ipc/ipc.sock";

const client = createConnection(IPC_PATH);

client.on("connect", (err) => {
	if (err) {
		console.error(err);
		return;
	}

	const bus = new EventEmitter();
	const extHandler = new ExtensionHostIPCHandler(bus);

	const channelMap = {};
	bus.on("request", (data) => {
		const channel = data?.channel;
		if (channel) {
			channelMap[channel] = true;
			client.write(`${JSON.stringify(data)}\n`);
		}
	});

	let oldBuffer: Buffer;

	client.on("data", async (data) => {
		let newData = Buffer.concat([oldBuffer ?? Buffer.alloc(0), data]);

		while (true) {
			const index = newData.findIndex((val) => val === "\n".charCodeAt(0));
			if (index === -1) {
				oldBuffer = newData;
				return;
			}

			const line = newData.subarray(0, index).toString();
			newData = newData.subarray(index + 1, newData.length);
			try {
				const parsed = JSON.parse(line.toString().trim());
				const channel = parsed?.channel;
				if (channel && channelMap[channel]) {
					console.log("got channel response", parsed);
					bus.emit(channel, parsed);
					continue;
				}

				const dataRet = await extHandler.parseMessage(parsed);

				const ret: mainHostMessage = {
					type: parsed.type,
					data: dataRet,
					channel: parsed.channel,
					extensionName: parsed.extensionName,
				};

				console.log("replying with ret", ret);
				client.write(`${JSON.stringify(ret)}\n`);
			} catch (e) {
				console.error(e);
			}
		}
	});
});
