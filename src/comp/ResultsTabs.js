import React from 'react';
import pt from 'prop-types';
import styled from '@emotion/styled';
import { Col, Row } from './Flex';

const Tabs = styled(Row)``;

const Tab = styled.button`
  padding: 0.4em 0.8em;
  color: ${(props) => (props.active ? 'var(--purple)' : 'var(--ij-fg)')};
  background: var(--ij-bg-alt);
  flex: 1 1 auto;
`;

const Content = styled(Col)`
  flex: 1 1 auto;

  & > * {
    height: 100%;
  }
  & > :not(:nth-child(${(props) => props.active + 1})) {
    color: red;
    display: none;
  }
`;

/**
 * This component
 */
function ResultsTabs(props) {
  const makeTab = (id, text) => (
    <Tab
      key={id}
      type="button"
      active={props.value === id}
      onMouseDown={(event) => {
        event.preventDefault();
        if (props.value !== id) {
          props.onChange(id);
        }
      }}
    >
      {text}
    </Tab>
  );

  const tabs = [makeTab('frame', 'Page'), makeTab('console', 'Console')];
  if (props.firstValue === 'console') {
    tabs.reverse();
  }

  return (
    <Col style={{ height: '100%' }}>
      <Tabs>{tabs}</Tabs>
      <Content active={tabs.findIndex((tab) => tab.props.active)}>
        {props.children}
      </Content>
    </Col>
  );
}

ResultsTabs.propTypes = {
  value: pt.oneOf(['frame', 'console']).isRequired,
  firstValue: pt.oneOf(['frame', 'console']).isRequired,
  onChange: pt.func.isRequire,
  children: pt.node.isRequire,
};

export default ResultsTabs;
