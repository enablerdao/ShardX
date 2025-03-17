import { ShardXClient } from './client';
import { Prediction, TradingPair } from './models';
/**
 * Price history point
 */
export interface PricePoint {
    /** Timestamp */
    timestamp: number;
    /** Price */
    price: string;
}
/**
 * AI prediction manager
 *
 * Utility class for working with AI predictions
 */
export declare class AIPredictionManager {
    private readonly client;
    /**
     * Create a new AI prediction manager
     * @param client ShardX client
     */
    constructor(client: ShardXClient);
    /**
     * Get available trading pairs
     * @returns Array of trading pairs
     */
    getTradingPairs(): Promise<TradingPair[]>;
    /**
     * Get prediction for a trading pair
     * @param pair Trading pair (e.g., "BTC/USD")
     * @param period Prediction period (e.g., "hour", "day", "week")
     * @returns Prediction
     */
    getPrediction(pair: string, period?: string): Promise<Prediction>;
    /**
     * Get predictions for multiple trading pairs
     * @param pairs Array of trading pairs
     * @param period Prediction period
     * @returns Map of predictions by pair
     */
    getPredictions(pairs: string[], period?: string): Promise<Map<string, Prediction>>;
    /**
     * Get price history for a trading pair
     * @param pair Trading pair
     * @param period Period (e.g., "hour", "day", "week", "month")
     * @param limit Maximum number of data points
     * @returns Array of price points
     */
    getPriceHistory(pair: string, period?: string, limit?: number): Promise<PricePoint[]>;
    /**
     * Calculate potential profit/loss
     * @param currentPrice Current price
     * @param predictedPrice Predicted price
     * @param investment Investment amount
     * @returns Potential profit/loss
     */
    calculatePotentialProfitLoss(currentPrice: string, predictedPrice: string, investment: string): {
        profitLoss: string;
        percentChange: string;
    };
    /**
     * Get trading recommendation
     * @param prediction Prediction
     * @returns Trading recommendation
     */
    getTradingRecommendation(prediction: Prediction): {
        action: 'buy' | 'sell' | 'hold';
        confidence: string;
        reasoning: string;
    };
}
