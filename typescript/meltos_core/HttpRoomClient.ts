import {SessionConfigs} from "./SessionConfigs";
import {RequestCommand, UserRequest} from "./Plugin";

const BASE_URI: string = "http://0.0.0.0:3000";

export class HttpRoomClient {
    constructor(
        readonly configs: SessionConfigs
    ) {
    }

    get roomId() {
        return this.configs.roomId;
    }

    get sessionId() {
        return this.configs.sessionId;
    }

    static async open() {
        const opened = await httpOpen();
        return new HttpRoomClient(opened);
    }

    static async join(roomId: string, userId?: string) {
        const response = await fetch(`${BASE_URI}/room/${roomId}/join`, {
            method: "POST",
            headers: {
                "content-type": "application/json",
            },
            body: JSON.stringify({
                user_id: userId,
            }),
        });
        const json = await response.json();
        const configs =  {
            roomId,
            sessionId: json["session_id"],
            userId: json["user_id"]
        };

        return new HttpRoomClient(configs);
    }

    readonly request = async (request: RequestCommand) => {
        await fetch(`${BASE_URI}/room/${this.roomId}/request`, {
            method: "POST",
            ...headers(this.sessionId),
            body: JSON.stringify(request)
        });
    };

    readonly leave = async () => {
        await fetch(`${BASE_URI}/room/${this.roomId}`, {
            method: "DELETE",
            ...headers(this.sessionId),
        });
    };

    private readonly apiUri = (uri?: string) => {
        return !!uri
            ? `${BASE_URI}/room/${this.roomId}/${uri}`
            : `${BASE_URI}/room/${this.roomId}`;
    };
}

export const httpOpen = async (userId?: string) => {
    const response = await fetch(`${BASE_URI}/room/open`, {
        method: "POST",
        headers: {
            "content-type": "application/json",
        },
        body: JSON.stringify({
            user_id: userId,
        }),
    });
    const json = await response.json();
    return {
        roomId: json["room_id"],
        sessionId: json["session_id"],
        userId: json["user_id"]
    };
};


const headers = (sessionId: string) => {
    return {
        headers: {
            "content-type": "application/json",
            "set-cookie": `session_id=${sessionId}`,
        },
    };
};
