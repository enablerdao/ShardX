import { ShardXClient } from './client';
import { Prediction, TradingPair } from './models';
import { ShardXError } from './errors';

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
export class AIPredictionManager {
  private readonly client: ShardXClient;

  /**
   * Create a new AI prediction manager
   * @param client ShardX client
   */
  constructor(client: ShardXClient) {
    this.client = client;
  }

  /**
   * Get available trading pairs
   * @returns Array of trading pairs
   */
  async getTradingPairs(): Promise<TradingPair[]> {
    return this.client.getTradingPairs();
  }

  /**
   * Get prediction for a trading pair
   * @param pair Trading pair (e.g., "BTC/USD")
   * @param period Prediction period (e.g., "hour", "day", "week")
   * @returns Prediction
   */
  async getPrediction(pair: string, period: string = 'hour'): Promise<Prediction> {
    return this.client.getPrediction(pair, period);
  }

  /**
   * Get predictions for multiple trading pairs
   * @param pairs Array of trading pairs
   * @param period Prediction period
   * @returns Map of predictions by pair
   */
  async getPredictions(pairs: string[], period: string = 'hour'): Promise<Map<string, Prediction>> {
    const predictions = new Map<string, Prediction>();
    
    // Get predictions in parallel
    const results = await Promise.allSettled(
      pairs.map(pair => this.getPrediction(pair, period))
    );
    
    // Process results
    results.forEach((result, index) => {
      const pair = pairs[index];
      
      if (result.status === 'fulfilled') {
        predictions.set(pair, result.value);
      } else {
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
  async getPriceHistory(pair: string, period: string = 'day', limit: number = 30): Promise<PricePoint[]> {
    try {
      const chartData = await this.client.getChartData('price', period, undefined, undefined);
      
      // Filter data for the requested pair
      const pairData = chartData.data.find((d: any) => d.pair === pair);
      
      if (!pairData) {
        return [];
      }
      
      // Convert to price points
      return pairData.points.slice(0, limit).map((point: any) => ({
        timestamp: point.timestamp,
        price: point.value
      }));
    } catch (error) {
      throw new ShardXError(
        `Failed to get price history: ${(error as Error).message}`,
        0,
        'price_history_error'
      );
    }
  }

  /**
   * Calculate potential profit/loss
   * @param currentPrice Current price
   * @param predictedPrice Predicted price
   * @param investment Investment amount
   * @returns Potential profit/loss
   */
  calculatePotentialProfitLoss(
    currentPrice: string,
    predictedPrice: string,
    investment: string
  ): { profitLoss: string; percentChange: string } {
    const current = parseFloat(currentPrice);
    const predicted = parseFloat(predictedPrice);
    const investmentAmount = parseFloat(investment);
    
    if (isNaN(current) || isNaN(predicted) || isNaN(investmentAmount)) {
      throw new ShardXError('Invalid input values', 400, 'invalid_input');
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
  getTradingRecommendation(prediction: Prediction): {
    action: 'buy' | 'sell' | 'hold';
    confidence: string;
    reasoning: string;
  } {
    const currentPrice = parseFloat(prediction.currentPrice);
    const predictedPrice = parseFloat(prediction.predictedPrice);
    const percentChange = ((predictedPrice - currentPrice) / currentPrice) * 100;
    const confidenceLevel = prediction.confidence;
    
    // Determine action based on price change and confidence
    let action: 'buy' | 'sell' | 'hold';
    let reasoning: string;
    
    if (percentChange > 5 && confidenceLevel > 0.7) {
      action = 'buy';
      reasoning = `Strong buy signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price increase of ${percentChange.toFixed(2)}%.`;
    } else if (percentChange < -5 && confidenceLevel > 0.7) {
      action = 'sell';
      reasoning = `Strong sell signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price decrease of ${Math.abs(percentChange).toFixed(2)}%.`;
    } else if (Math.abs(percentChange) < 2 || confidenceLevel < 0.5) {
      action = 'hold';
      reasoning = `Hold recommendation due to small predicted change (${percentChange.toFixed(2)}%) or low confidence (${confidenceLevel.toFixed(2)}).`;
    } else if (percentChange > 0) {
      action = 'buy';
      reasoning = `Moderate buy signal with ${confidenceLevel.toFixed(2)} confidence. Predicted price increase of ${percentChange.toFixed(2)}%.`;
    } else {
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