import {RoomClient} from "./RoomClient";
import {UserRequest, RoomCommand, RoomCommandPlugin, RoomOwnerPlugin} from "./Plugin";
import {sleep} from "./sleep";


class Echo implements RoomCommandPlugin, RoomOwnerPlugin {
    async onRequest(request: UserRequest): Promise<RoomCommand | RoomCommand[] | null> {
        console.log(request)
        switch (request.data.name) {
            case "echo":
                return {
                    to: [request.from],
                    name: "echo",
                    data: request.data
                };
            default:
                return null;
        }
    }

    async execute(command: RoomCommand): Promise<void> {

        switch (command.name) {
            case "echo":
                console.log(command.data);
                break;
            default:
                break;
        }
    }
}

(async () => {
    const client = await RoomClient.open(
        [new Echo()],
        [new Echo()]
    );
    console.log(client.configs);

    await sleep();
})()

