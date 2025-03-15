from typing import List, Dict, Optional, Any
from dataclasses import dataclass

from .client import ShardXClient
from .models import Prediction, TradingPair
from .errors import ShardXError

@dataclass
class PricePoint:
    """Price history point"""
    timestamp: int
    price: str

@dataclass
class TradingRecommendation:
    """Trading recommendation"""
    action: str  # "buy", "sell", or "hold"
    confidence: str
    reasoning: str

@dataclass
class ProfitLossResult:
    """Profit/loss calculation result"""
    profit_loss: str
    percent_change: str

class AIPredictionManager:
    """
    AI prediction manager
    
    Utility class for working with AI predictions
    """
    
    def __init__(self, client: ShardXClient):
        """
        Initialize a new AI prediction manager
        
        Args:
            client: ShardX client
        """
        self.client = client
    
    async def get_trading_pairs(self) -> List[TradingPair]:
        """
        Get available trading pairs
        
        Returns:
            List of trading pairs
        """
        return self.client.get_trading_pairs()
    
    async def get_prediction(self, pair: str, period: str = "hour") -> Prediction:
        """
        Get prediction for a trading pair
        
        Args:
            pair: Trading pair (e.g., "BTC/USD")
            period: Prediction period (e.g., "hour", "day", "week")
            
        Returns:
            Prediction
        """
        return self.client.get_prediction(pair, period)
    
    async def get_predictions(
        self,
        pairs: List[str],
        period: str = "hour"
    ) -> Dict[str, Prediction]:
        """
        Get predictions for multiple trading pairs
        
        Args:
            pairs: List of trading pairs
            period: Prediction period
            
        Returns:
            Dictionary of predictions by pair
        """
        predictions = {}
        
        for pair in pairs:
            try:
                prediction = await self.get_prediction(pair, period)
                predictions[pair] = prediction
            except Exception as e:
                print(f"Failed to get prediction for {pair}: {e}")
        
        return predictions
    
    async def get_price_history(
        self,
        pair: str,
        period: str = "day",
        limit: int = 30
    ) -> List[PricePoint]:
        """
        Get price history for a trading pair
        
        Args:
            pair: Trading pair
            period: Period (e.g., "hour", "day", "week", "month")
            limit: Maximum number of data points
            
        Returns:
            List of price points
        """
        try:
            chart_data = self.client.get_chart_data("price", period)
            
            # Filter data for the requested pair
            pair_data = next((d for d in chart_data.get("data", []) if d.get("pair") == pair), None)
            
            if not pair_data:
                return []
            
            # Convert to price points
            return [
                PricePoint(
                    timestamp=point.get("timestamp"),
                    price=str(point.get("value"))
                )
                for point in pair_data.get("points", [])[:limit]
            ]
        except Exception as e:
            raise ShardXError(
                f"Failed to get price history: {str(e)}",
                0,
                "price_history_error"
            )
    
    def calculate_potential_profit_loss(
        self,
        current_price: str,
        predicted_price: str,
        investment: str
    ) -> ProfitLossResult:
        """
        Calculate potential profit/loss
        
        Args:
            current_price: Current price
            predicted_price: Predicted price
            investment: Investment amount
            
        Returns:
            Potential profit/loss
            
        Raises:
            ShardXError: If input values are invalid
        """
        try:
            current = float(current_price)
            predicted = float(predicted_price)
            investment_amount = float(investment)
        except ValueError:
            raise ShardXError("Invalid input values", 400, "invalid_input")
        
        percent_change = ((predicted - current) / current) * 100
        profit_loss = investment_amount * (percent_change / 100)
        
        return ProfitLossResult(
            profit_loss=f"{profit_loss:.2f}",
            percent_change=f"{percent_change:.2f}"
        )
    
    def get_trading_recommendation(self, prediction: Prediction) -> TradingRecommendation:
        """
        Get trading recommendation
        
        Args:
            prediction: Prediction
            
        Returns:
            Trading recommendation
        """
        current_price = float(prediction.current_price)
        predicted_price = float(prediction.predicted_price)
        percent_change = ((predicted_price - current_price) / current_price) * 100
        confidence_level = prediction.confidence
        
        # Determine action based on price change and confidence
        action: str
        reasoning: str
        
        if percent_change > 5 and confidence_level > 0.7:
            action = "buy"
            reasoning = f"Strong buy signal with {confidence_level:.2f} confidence. Predicted price increase of {percent_change:.2f}%."
        elif percent_change < -5 and confidence_level > 0.7:
            action = "sell"
            reasoning = f"Strong sell signal with {confidence_level:.2f} confidence. Predicted price decrease of {abs(percent_change):.2f}%."
        elif abs(percent_change) < 2 or confidence_level < 0.5:
            action = "hold"
            reasoning = f"Hold recommendation due to small predicted change ({percent_change:.2f}%) or low confidence ({confidence_level:.2f})."
        elif percent_change > 0:
            action = "buy"
            reasoning = f"Moderate buy signal with {confidence_level:.2f} confidence. Predicted price increase of {percent_change:.2f}%."
        else:
            action = "sell"
            reasoning = f"Moderate sell signal with {confidence_level:.2f} confidence. Predicted price decrease of {abs(percent_change):.2f}%."
        
        return TradingRecommendation(
            action=action,
            confidence=f"{confidence_level:.2f}",
            reasoning=reasoning
        )