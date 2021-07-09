import React, { useState } from 'react';
import pt from 'prop-types';
import styled from '@emotion/styled';
import { Col, Row } from './Flex';
import { EventType } from '../EventType';

const splitBasis = (percent, vertical) => {
  const shared = { flexShrink: 1000 };
  const prop = vertical ? 'width' : 'height';

  const toSize = (p2) => `calc(${p2 * 100}% - 1px)`;

  return {
    a: { [prop]: toSize(percent), ...shared },
    b: { [prop]: toSize(1 - percent), ...shared },
  };
};

const Cell = styled.section`
  position: relative;
  border: 1px solid var(--ij-fg);
`;

const DividerX = styled.button`
  appearance: none;
  display: block;
  position: absolute;
  top: 0;
  bottom: 0;
  left: calc(${(props) => props.percent * 100}% - var(--quad-split-divider) / 2);
  width: var(--quad-split-divider);
  background: transparent;
  /* background: blue; */

  &:hover {
    background: rgba(255, 255, 255, 0.5);
  }
`;

DividerX.propTypes = {
  percent: pt.number.isRequired,
};

const DividerY = styled.button`
  appearance: none;
  display: block;
  position: absolute;
  top: calc(${(props) => props.percent * 100}% - var(--quad-split-divider) / 2);
  width: 100%;
  left: 0;
  right: 0;
  height: var(--quad-split-divider);
  background: transparent;
  /* background: red; */

  &:hover {
    background: rgba(255, 255, 255, 0.5);
  }
`;

DividerY.propTypes = {
  percent: pt.number.isRequired,
};

const Net = styled.div`
  display: block;
  position: fixed;
  top: -1000px;
  right: -1000px;
  bottom: -1000px;
  left: -1000px;
  z-index: 100000;
  // background: rgba(255, 255, 255, 0.2);
`;

function Divider({ vertical, percent, isSizing, onPress, onRelease, onChange }) {
  const start = React.useRef(null);

  const Tag = vertical ? DividerX : DividerY;

  const getLength = (event) => (vertical ? event.pageX : event.pageY);
  const getSize = (event) => {
    const box = event.currentTarget.parentElement.getBoundingClientRect();
    return vertical ? box.width : box.height;
  };

  return (
    <Tag
      percent={percent}
      onMouseDown={(event) => {
        start.current = { length: getLength(event), size: getSize(event), percent };
        onPress();
      }}
      onMouseUp={() => {
        start.current = null;
        onRelease();
      }}
      onMouseMove={(event) => {
        if (!start.current || !isSizing) {
          return;
        }

        const length = getLength(event);
        const delta = length - start.current.length;
        onChange(start.current.percent + delta / start.current.size);
      }}
    >
      {isSizing && <Net />}
    </Tag>
  );
}

Divider.propTypes = {
  vertical: pt.bool,
  isSizing: pt.bool,
  onPress: pt.func.isRequired,
  onRelease: pt.func.isRequired,
  onChange: pt.func.isRequired,
  percent: pt.number.isRequired,
};

function useSplit({ ident, vertical, initial, sizing, setSizing }) {
  const [percent, setPercent] = React.useState(initial);

  return {
    value: percent,
    basis: splitBasis(percent, vertical),
    set: setPercent,
    sizing,
    setSizing,
    divider: (
      <Divider
        vertical={vertical}
        percent={percent}
        isSizing={sizing === ident}
        onPress={() => {
          setSizing(ident);
        }}
        onRelease={() => {
          setSizing(null);
        }}
        onChange={(percent) => {
          setPercent(percent);
        }}
      />
    ),
  };
}

function useSplits(resize) {
  const [sizing, setSizing] = useState(null);

  const x = useSplit({
    ident: 'x',
    vertical: true,
    initial: 0.5,
    sizing,
    setSizing,
  });
  const y1 = useSplit({
    ident: 'y1',
    vertical: false,
    initial: 0.5,
    sizing,
    setSizing,
  });
  const y2 = useSplit({
    ident: 'y2',
    vertical: false,
    initial: 0.5,
    sizing,
    setSizing,
  });

  const queued = React.useRef(false);
  React.useEffect(() => {
    if (!queued.current) {
      queued.current = true;

      requestAnimationFrame(() => {
        queued.current = false;
        resize.emit();
      });
    }
  }, [sizing]);

  return { x, y1, y2 };
}

const Col2 = styled(Col)`
  position: relative;
`;

const QuadBox = styled(Row)`
  height: calc(100% - 2px);
  position: relative;

  & > button,
  & > * > button {
    padding: 0;
    border: none;
    --quad-split-divider: 8px;
  }
`;

const Center = styled.aside`
  position: absolute;
  top: ${(props) => props.y * 100 + '%'};
  left: ${(props) => props.x * 100 + '%'};
  transform: translate(-50%, -50%);
  z-index: 1000;
`;

function QuadSplit(props) {
  const splits = useSplits(props.resize);
  const { children } = props;

  if (!Array.isArray(children)) {
    throw new Error(`QuadSplit: Expected props.children to be an array`);
  }
  if (children.length !== 4) {
    throw new Error(
      `QuadSplit: Expected props.children.length to be 4 but got ${props.children.length}`,
    );
  }

  const centerX = splits.x.value;
  const diff1 = Math.abs(splits.y1.value - 0.5);
  const diff2 = Math.abs(splits.y2.value - 0.5);
  const centerY = diff1 < diff2 ? splits.y1.value : splits.y2.value;

  return (
    <QuadBox
      horizontal="stretch"
      onKeyPress={(event) => {
        const ctrl = event.ctrlKey || event.metaKey;
        if (ctrl && event.key === 'Enter' && props.onSubmit) {
          props.onSubmit();
        }
      }}
    >
      <Col2 style={splits.x.basis.a}>
        <Cell style={splits.y1.basis.a}>{children[0]}</Cell>
        <Cell style={splits.y1.basis.b}>{children[1]}</Cell>
        {splits.y1.divider}
      </Col2>
      <Col2 style={splits.x.basis.b}>
        <Cell style={splits.y2.basis.a}>{children[2]}</Cell>
        <Cell style={{ ...splits.y2.basis.b, overflow: 'hidden' }}>
          {children[3]}
        </Cell>

        {splits.y2.divider}
      </Col2>
      {splits.x.divider}

      {props.center && (
        <Center x={centerX} y={centerY}>
          {props.center()}
        </Center>
      )}
    </QuadBox>
  );
}

QuadSplit.propTypes = {
  resize: pt.instanceOf(EventType).isRequired,
  children: pt.arrayOf(pt.node).isRequired,
  onSubmit: pt.func,
  center: pt.func,
};

export default QuadSplit;
