import React, { useState } from 'react';
import styled from '@emotion/styled';
import { Col, Row } from './Flex';

const splitBasis = (percent) => {
  const SCALE = 10000;
  const p = Math.floor(percent * SCALE);
  const shared = { flexGrow: 1, flexShrink: 1 };
  return {
    a: { flexBasis: String(p), ...shared },
    b: { flexBasis: String(SCALE - p), ...shared },
  };
};

const Cell = styled.section`
  border: 1px solid transparent;

  *:nth-of-type(1) > &:nth-of-type(1) {
    border-right-color: var(--ij-fg);
    border-bottom-color: var(--ij-fg);
  }
  *:nth-of-type(2) > &:nth-of-type(1) {
    border-bottom-color: var(--ij-fg);
  }
  *:nth-of-type(2) > &:nth-of-type(2) {
    border-left-color: var(--ij-fg);
  }
`;

function QuadSplit(props) {
  const [x, setX] = useState(props.initialX || 0.5);
  const [y1, setY1] = useState(props.initialY1 || 0.5);
  const [y2, setY2] = useState(props.initialY2 || 0.5);

  const basisRow = splitBasis(x);
  const basisLeft = splitBasis(y1);
  const basisRight = splitBasis(y2);

  return (
    <Row horizontal="stretch" style={{ height: '100%' }}>
      <Col style={basisRow.a}>
        <Cell style={basisLeft.a}>C1</Cell>
        <Cell style={basisLeft.b}>C2</Cell>
      </Col>
      <Col style={basisRow.b}>
        <Cell style={basisRight.a}>C3</Cell>
        <Cell style={basisRight.b}>C4</Cell>
      </Col>
    </Row>
  );
}

export default QuadSplit;
