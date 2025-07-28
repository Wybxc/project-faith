import { CardServiceClientImpl, GrpcWebImpl } from "../generated/proto/card.v1";
import { HOST } from "./common";

export class CardV1Api {
    private rpc: GrpcWebImpl
    private client: CardServiceClientImpl

    constructor() {
        this.rpc = new GrpcWebImpl(HOST, {});
        this.client = new CardServiceClientImpl(this.rpc);
    }

    async getCardPrototypes() {
        const response = await this.client.GetCardPrototypes({});
        return response.prototypes;
    }
}