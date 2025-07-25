import { grpc } from "@improbable-eng/grpc-web";
import { GameServiceClientImpl, GrpcWebImpl } from "../generated/proto/game.v1";
import { HOST } from "./common";

export class GameV1Api {
    private rpc: GrpcWebImpl;
    private client: GameServiceClientImpl;
    private roomName: string | null = null;
    private roomId: string | null = null;

    constructor(token: string) {
        const metadata = new grpc.Metadata();
        metadata.append('authentication', `Bearer ${token}`);

        this.rpc = new GrpcWebImpl(HOST, { metadata });
        this.client = new GameServiceClientImpl(this.rpc);
    }

    getRoomName() {
        return this.roomName ?? "unknown";
    }

    async joinRoom(roomName: string) {
        const response = await this.client.JoinRoom({ roomName });
        if (!response.success) {
            throw new Error(response.message);
        }
        this.roomName = roomName;
        this.roomId = response.roomId;
        return response.message;
    }

    enterGame() {
        if (!this.roomId) {
            throw new Error("You must join a room before entering the game.");
        }
        const response = this.client.EnterGame({ roomId: this.roomId! });
        return response;
    }

    async ping() {
        if (!this.roomId) {
            throw new Error("You must join a room before pinging.");
        }
        console.log(`Pinging room: ${this.roomId}`);
        const response = await this.client.Ping({ roomId: this.roomId! });
        return response;
    }
}