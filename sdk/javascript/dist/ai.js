"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AIPredictionManager = void 0;
const errors_1 = require("./errors");
/**
 * AI prediction manager
 *
 * Utility class for working with AI predictions
 */
class AIPredictionManager {
    /**
     * Create a new AI prediction manager
     * @param client ShardX client
     */
    constructor(client) {
        this.client = client;
    }
    /**
     * Get available trading pairs
     * @returns Array of trading pairs
     */
    async getTradingPairs() {
        return this.client.getTradingPairs();
    }
    /**
     * Get prediction for a trading pair
     * @param pair Trading pair (e.g., "BTC/USD")
     * @param period Prediction period (e.g., "hour", "day", "week")
     * @returns Prediction
     */
    async getPrediction(pair, period = 'hour') {
        return this.client.getPrediction(pair, period);
    }
    /**
     * Get predictions for multiple trading pairs
     * @param pairs Array of trading pairs
     * @param period Prediction period
     * @returns Map of predictions by pair
     */
    async getPredictions(pairs, period = 'hour') {
        const predictions = new Map();
        // Get predictions in parallel
        const results = await Promise.allSettled(pairs.map(pair => this.getPrediction(pair, period)));
        // Process results
        results.forEach((result, index) => {
            const pair = pairs[index];
            if (result.status === 'fulfilled') {
                predictions.set(pair, result.value);
            }
            else {
                console.error(`Failed to get prediction for ${pair}: ${result.reason}`);
            }
        });
        return predictions;
    }
    /**
     * Get price history for a trading pair
     * @param pair Trading pair
     * @param period Period (e.g., "hour", "day", "week", "month")
     * @param limit Maximum number of data points
     * @returns Array of price points
     */
    async getPriceHistory(pair, period = 'day', limit = 30) {
        try {
            const chartData = await this.client.getChartData('price', period, undefined, undefined);
            // Filter data for the requested pair
            const pairData = chartData.data.find((d) => d.pair === pair);
            if (!pairData) {
                return [];
            }
            // Convert to price points
            return pairData.points.slice(0, limit).map((point) => ({
                timestamp: point.timestamp,
                price: point.value
            }));
        }
        catch (error) {
            throw new errors_1.ShardXError(`Failed to get price history: ${error.message}`, 0, 'price_history_error');
        }
    }
    /**
     * Calculate potential profit/loss
     * @param currentPrice Current price
     * @param predictedPrice Predicted price
     * @param investment Investment amount
     * @returns Potential profit/loss
     */
    calculatePotentialProfitLoss(currentPrice, predictedPrice, investment) {
        const current = parseFloat(currentPrice);
        const predicted = parseFloat(predictedPrice);
        const investmentAmount = parseFloat(investment);
        if (isNaN(current) || isNaN(predicted) || isNaN(investmentAmount)) {
            throw new errors_1.ShardXError('Invalid input values', 400, 'invalid_input');
        }
        const percentChange = ((predicted - current) / current) * 100;
        const profitLoss = investmentAmount * (percentChange / 100);
        return {
            profitLoss: profitLoss.toFixed(2),
            percentChange: percentChange.toFixed(2)
        };
    }
    /**
     * Get trading recommendation
     * @param prediction Prediction
     * @returns Trading recommendation
     */
    getTradingRecommendation(prediction) {
        const currentPrice = parseFloat(prediction.currentPrice);
        const predictedPrice = parseFloat(prediction.predictedPrice);
        const percentChange = ((predictedPrice - currentPrice) / currentPrice) * 100;
        const confidenceLevel = prediction.confidence;
        // Determine action based on price change and confidence
        let action;
        let reasoning;
        if (percentChange > 5 && confidenceLevel > 0.7) {
            action = 'buy';
            reasoning = `Strong buy signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price increase of ${percentChange.toFixed(2)}%.`;
        }
        else if (percentChange < -5 && confidenceLevel > 0.7) {
            action = 'sell';
            reasoning = `Strong sell signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price decrease of ${Math.abs(percentChange).toFixed(2)}%.`;
        }
        else if (Math.abs(percentChange) < 2 || confidenceLevel < 0.5) {
            action = 'hold';
            reasoning = `Hold recommendation due to small predicted change (${percentChange.toFixed(2)}%) or low confidence (${confidenceLevel.toFixed(2)}).`;
        }
        else if (percentChange > 0) {
            action = 'buy';
            reasoning = `Moderate buy signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price increase of ${percentChange.toFixed(2)}%.`;
        }
        else {
            action = 'sell';
            reasoning = `Moderate sell signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price decrease of ${Math.abs(percentChange).toFixed(2)}%.`;
        }
        return {
            action,
            confidence: confidenceLevel.toFixed(2),
            reasoning
        };
    }
}
exports.AIPredictionManager = AIPredictionManager;
//# sourceMappingURL=ai.js.map