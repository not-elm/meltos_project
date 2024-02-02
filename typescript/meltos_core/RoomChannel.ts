import WebSocket from "ws";
import {RoomCommandPlugin, RoomOwnerPlugin} from "./Plugin";
import {SessionConfigs} from "./SessionConfigs";

export class RoomChannel implements Disposable {
    private constructor(
        private readonly configs: SessionConfigs,
        private readonly ws: WebSocket
    ) {
    }

    static connect(
        configs: SessionConfigs,
        ownerPlugins: RoomOwnerPlugin[],
        plugins: RoomCommandPlugin[],
    ): Promise<RoomChannel> {
        let completed: boolean = false;
        return new Promise((resolve, reject) => {
            const ws = new WebSocket(`ws://0.0.0.0:3000/room/${configs.roomId}/channel`, {
                headers: {
                    "set-cookie": `session_id=${configs.sessionId}`,
                },
            });
            ws.onopen = () => {
                if (!completed) {
                    completed = true;
                    ws.onmessage = (message) => {
                        const command = message.data;

                        if (typeof command === "string") {
                            const json = JSON.parse(command)
                            switch (json["type"]) {
                                case "request":
                                    ownerPlugins.forEach(async p => {
                                        await p.onRequest(JSON.parse(json["data"]));
                                    });
                                    break;
                                case "command":
                                    plugins.forEach(async p => {
                                        await p.execute(json["data"]);
                                    });
                                    break;
                                default:
                                    break;
                            }

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
    }


    [Symbol.dispose](): void {
        this.ws.close();
    }
}


