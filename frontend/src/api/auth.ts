import { AuthServiceClientImpl, GrpcWebImpl } from "../generated/proto/auth.v1";
import { HOST } from "./common";

export class AuthV1Api {
    private rpc: GrpcWebImpl
    private client: AuthServiceClientImpl

    constructor() {
        this.rpc = new GrpcWebImpl(HOST, {});
        this.client = new AuthServiceClientImpl(this.rpc);
    }

    async login(username: string) {
        const response = await this.client.Login({ username });
        return response.token;
    }
}