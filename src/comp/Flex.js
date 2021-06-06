import React, { useState } from 'react';
import styled from '@emotion/styled';

export const Row = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: ${(props) => props.horizontal};
  align-items: ${(props) => props.vertical};
`;

export const Col = styled.div`
  display: flex;
  flex-direction: column;
  justify-items: ${(props) => props.vertical};
  align-content: ${(props) => props.horizontal};
`;
