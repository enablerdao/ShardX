import React from 'react';
import { Container, Row, Col } from 'react-bootstrap';
import { useShardX } from '../contexts/ShardXContext';

const Footer = () => {
  const { nodeInfo } = useShardX();
  
  return (
    <footer className="bg-light py-4 mt-auto">
      <Container>
        <Row className="align-items-center">
          <Col md={6} className="text-center text-md-start">
            <p className="mb-0 text-muted">
              &copy; {new Date().getFullYear()} ShardX DeFi. All rights reserved.
            </p>
          </Col>
          <Col md={6} className="text-center text-md-end">
            {nodeInfo && (
              <p className="mb-0 text-muted">
                <small>
                  Connected to ShardX v{nodeInfo.version} | 
                  Block Height: {nodeInfo.height} | 
                  Peers: {nodeInfo.peers}
                </small>
              </p>
            )}
          </Col>
        </Row>
      </Container>
    </footer>
  );
};

export default Footer;