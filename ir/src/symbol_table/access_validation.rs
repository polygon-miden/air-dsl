use super::{
    ConstantType, MatrixAccess, NamedTraceAccess, SemanticError, Symbol, SymbolType, VariableType,
    VectorAccess,
};

/// Checks that the specified access into an identifier is valid and returns an error otherwise.
/// # Errors:
/// - Returns an error if the type of the identifier does not allow the access type. For example,
///   VariableType::Vector does not allow a MatrixAccess.
/// - Returns an error if any indices specified for the access are out of bounds fo the specified
///   identifier.
pub(super) trait ValidateIdentifierAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError>;
}

impl ValidateIdentifierAccess for NamedTraceAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        match symbol.symbol_type() {
            SymbolType::TraceColumns(columns) => {
                if self.idx() >= columns.size() {
                    return Err(SemanticError::named_trace_column_access_out_of_bounds(
                        self,
                        columns.size(),
                    ));
                }
            }
            _ => {
                return Err(SemanticError::not_a_trace_column_identifier(
                    symbol.name(),
                    symbol.symbol_type(),
                ))
            }
        }

        Ok(())
    }
}

/// Checks that the specified vector access is valid and returns an error otherwise.
impl ValidateIdentifierAccess for VectorAccess {
    /// TODO: docs (errors)
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        let vector_len = match symbol.symbol_type() {
            SymbolType::Constant(ConstantType::Vector(vector)) => vector.len(),
            SymbolType::PublicInput(size) => *size,
            SymbolType::RandomValuesBinding(_, size) => *size,
            SymbolType::TraceColumns(trace_columns) => trace_columns.size(),
            SymbolType::Variable(variable) => {
                match variable {
                    // TODO: scalar can be ok; check this symbol in the future
                    VariableType::Scalar(_) => return Ok(()),
                    VariableType::Vector(vector) => vector.len(),
                    _ => {
                        return Err(SemanticError::invalid_vector_access(
                            self,
                            symbol.symbol_type(),
                        ))
                    }
                }
            }
            _ => {
                return Err(SemanticError::invalid_vector_access(
                    self,
                    symbol.symbol_type(),
                ))
            }
        };

        if self.idx() >= vector_len {
            return Err(SemanticError::vector_access_out_of_bounds(self, vector_len));
        }

        Ok(())
    }
}

/// Checks that the specified matrix access is valid and returns an error otherwise.
impl ValidateIdentifierAccess for MatrixAccess {
    /// TODO: docs (errors)
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        let (row_len, col_len) = match symbol.symbol_type() {
            SymbolType::Constant(ConstantType::Matrix(matrix)) => (matrix.len(), matrix[0].len()),

            SymbolType::Variable(variable) => {
                match variable {
                    // TODO: scalar & vector can be ok; check this symbol in the future
                    VariableType::Scalar(_) | VariableType::Vector(_) => return Ok(()),
                    VariableType::Matrix(matrix) => (matrix.len(), matrix[0].len()),
                    _ => {
                        return Err(SemanticError::invalid_matrix_access(
                            self,
                            symbol.symbol_type(),
                        ))
                    }
                }
            }
            _ => {
                return Err(SemanticError::invalid_matrix_access(
                    self,
                    symbol.symbol_type(),
                ))
            }
        };

        if self.row_idx() >= row_len || self.col_idx() >= col_len {
            return Err(SemanticError::matrix_access_out_of_bounds(
                self, row_len, col_len,
            ));
        }

        Ok(())
    }
}