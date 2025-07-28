interface Card {
    name: string;
    description: string;
}

const cards: Record<number, Card | undefined> = {
    7001: {
        name: "测试卡牌7001",
        description: "抽一张牌。"
    },
    7002: {
        name: "测试卡牌7002",
        description: "抽两张牌。"
    },
    8001: {
        name: "信念8001",
        description: "这是一个信念卡牌。"
    }
};

export default cards;
