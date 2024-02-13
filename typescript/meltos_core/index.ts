import {HttpRoomClient} from "./HttpRoomClient";
import {RoomChannel} from "./RoomChannel";
import {RequestCommand, RoomCommandPlugin, RoomOwnerPlugin} from "./Plugin";
import {SessionConfigs} from "./SessionConfigs";

export class RoomClient {
    private constructor(
        readonly configs: SessionConfigs,
        private readonly http: HttpRoomClient,
        private readonly channel: RoomChannel,
    ) {
    }

    static async open(
        ownerPlugins: RoomOwnerPlugin[],
        plugins: RoomCommandPlugin[]
    ) {
        const http = await HttpRoomClient.open()
        const channel = await RoomChannel.open(http.configs, ownerPlugins, plugins, async commands => {
            await http.command(commands)
        });
        return new RoomClient(http.configs, http, channel);
    }

    static async join(
        roomId: string,
        plugins: RoomCommandPlugin[]
    ) {
        const http = await HttpRoomClient.join(roomId)
        const channel = await RoomChannel.join(http.configs, plugins);
        return new RoomClient(http.configs, http, channel);
    }

    async request(req: RequestCommand) {
        await this.http.request(req)
    }

}
