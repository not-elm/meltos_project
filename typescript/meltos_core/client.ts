import {RoomCommand, RoomCommandPlugin, RoomOwnerPlugin, UserRequest} from "./Plugin";
import {RoomClient} from "./RoomClient";
import {sleep} from "./sleep";

class Echo implements RoomCommandPlugin, RoomOwnerPlugin {
    async onRequest(request: UserRequest): Promise<RoomCommand | RoomCommand[] | null> {
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
    const client = await RoomClient.join(
        "d17e48a9cdf9d842dcc5a2c5335a9d561c341231",
        [new Echo()]
    );
    console.log(client.configs);

    setInterval(() => {
        console.log("interval")

        client.request({
            name: "echo",
            data: "hello!"
        })
    }, 1000);
    await sleep();
})()