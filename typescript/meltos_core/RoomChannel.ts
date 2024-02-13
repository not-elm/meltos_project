import WebSocket from "ws";
import { RoomCommand, RoomCommandPlugin, RoomOwnerPlugin } from "./Plugin";
import { SessionConfigs } from "./SessionConfigs";

export class RoomChannel implements Disposable {
	private constructor(
		private readonly configs: SessionConfigs,
		private readonly ws: WebSocket
	) {}

	static open(
		configs: SessionConfigs,
		ownerPlugins: RoomOwnerPlugin[],
		commandPlugins: RoomCommandPlugin[],
        onResponse: (commands: RoomCommand[]) => Promise<void>
	): Promise<RoomChannel> {
		return RoomChannel.connectWs(configs, async (json) => {
			switch (json["type"]) {
				case "request":
					const commands: RoomCommand[] = [];
                    ownerPlugins.forEach(async (p) => {
						const responses = await p.onRequest(JSON.parse(json["data"]));
                        if(responses){
                            if(Array.isArray(responses)){
                                commands.push(...responses);
                            }else{
                                commands.push(responses);
                            }
                        }
                    });
                    if(0 < commands.length){
                        await onResponse(commands);
                    }
					break;
				case "command":
					commandPlugins.forEach(async (p) => {
						await p.execute(json["data"]);
					});
					break;
				default:
					break;
			}
		});
	}

	static join(
		configs: SessionConfigs,
		commandPlugins: RoomCommandPlugin[]
	): Promise<RoomChannel> {
		return RoomChannel.connectWs(configs, async (json) => {
			switch (json["type"]) {
				case "command":
					commandPlugins.forEach(async (p) => {
						await p.execute(json["data"]);
					});
					break;
				default:
					break;
			}
		});
	}

	private static connectWs = (
		configs: SessionConfigs,
		onMessage: (message: any) => void
	): Promise<RoomChannel> => {
		let completed: boolean = false;
		return new Promise((resolve, reject) => {
			const ws = new WebSocket(
				`ws://0.0.0.0:3000/room/${configs.roomId}/channel`,
				{
					headers: {
						"set-cookie": `session_id=${configs.sessionId}`,
					},
				}
			);
			ws.onopen = () => {
				if (!completed) {
					completed = true;
					ws.onmessage = (message) => {
						const messageText = message.data;
						if (typeof messageText === "string") {
							onMessage(JSON.parse(messageText));
						}
					};
					resolve(new RoomChannel(configs, ws));
				}
			};
			ws.onerror = (e) => {
				if (!completed) {
					completed = true;
					reject(e);
				}
			};
		});
	};

	[Symbol.dispose](): void {
		this.ws.close();
	}
}
