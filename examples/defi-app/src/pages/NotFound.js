import React from 'react';
import { Container, Button } from 'react-bootstrap';
import { Link } from 'react-router-dom';

const NotFound = () => {
  return (
    <Container>
      <div className="not-found">
        <div className="not-found-icon">
          <i className="bi bi-exclamation-triangle"></i>
        </div>
        <h1 className="not-found-title">404 - Page Not Found</h1>
        <p className="not-found-message">
          The page you are looking for does not exist or has been moved.
        </p>
        <Link to="/">
          <Button variant="gradient" size="lg">
            Go to Dashboard
          </Button>
        </Link>
      </div>
    </Container>
  );
};

export default NotFound;