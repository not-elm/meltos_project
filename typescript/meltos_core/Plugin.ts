export interface RoomCommand {
    to: string[] | null
    name: string,
    data: any
}

export interface UserRequest {
    from: string
    data: RequestCommand
}

export interface RequestCommand {
    name: string
    data: any
}

export interface RoomOwnerPlugin {
    onRequest(request: UserRequest): Promise<RoomCommand | RoomCommand[] | null>
}

export interface RoomCommandPlugin {
    execute(command: RoomCommand): Promise<void>
}
