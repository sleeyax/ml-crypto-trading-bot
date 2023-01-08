import numpy as np
import os
import LSTM_model_builder as model_builder

maindf = model_builder.load_dataset("datasets/BTC-Hourly-Binance.csv")
closedf = model_builder.get_prediction_dataframe(maindf)
closedf = model_builder.normalize(closedf)

train_data, test_data = model_builder.split_dataset(closedf)

time_step = 15
X_train, y_train = model_builder.create_dataset_matrix(train_data, time_step)
X_test, y_test = model_builder.create_dataset_matrix(test_data, time_step)

print("X_train shape: ", X_train.shape)
print("y_train shape: ", y_train.shape)
print("X_test shape: ", X_test.shape)
print("y_test shape", y_test.shape)

X_train = model_builder.normalize_lstm(X_train)
X_test = model_builder.normalize_lstm(X_test)

print("X_train: ", X_train.shape)
print("X_test: ", X_test.shape)

# load or create the model
model_path = 'btc_predict_closed_model.h5'
if os.path.exists(model_path):
  model = model_builder.load_model(model_path)
else:
  model = model_builder.create_model()
  model = model_builder.train_model(model, X_train, y_train, X_test, y_test)
  model.save(model_path, save_format='h5')

# use the model to predict values based on the trained data
train_predict = model.predict(X_train)
print("predictions trained shape: ", train_predict.shape)

# use the model predict values based on (unseen) testing data
test_predict = model.predict(X_test)
print("predictions test shape:", test_predict.shape)

# transform back to original form
train_predict = model_builder.unnormalize(train_predict)
test_predict = model_builder.unnormalize(test_predict)

# evaluate model performance
model_builder.evaluate_performance(train_predict, test_predict, y_train, y_test)

# Predict next 30 days
x_input = test_data[len(test_data) - time_step:].reshape(1,-1)
temp_input = list(x_input)
temp_input = temp_input[0].tolist()

lst_output=[]
n_steps=time_step
i=0
pred_days = 30
while(i < pred_days):
    
    if(len(temp_input)>time_step):
        
        x_input=np.array(temp_input[1:])
        #print("{} day input {}".format(i,x_input))
        x_input = x_input.reshape(1,-1)
        x_input = x_input.reshape((1, n_steps, 1))
        
        yhat = model.predict(x_input, verbose=0)
        #print("{} day output {}".format(i,yhat))
        temp_input.extend(yhat[0].tolist())
        temp_input=temp_input[1:]
        #print(temp_input)
       
        lst_output.extend(yhat.tolist())
        i=i+1
        
    else:
        
        x_input = x_input.reshape((1, n_steps,1))
        yhat = model.predict(x_input, verbose=0)
        temp_input.extend(yhat[0].tolist())
        
        lst_output.extend(yhat.tolist())
        i=i+1

lstmdf=(np.array(lst_output).reshape(-1,1)).tolist()
lstmdf=model_builder.unnormalize(lstmdf).reshape(1,-1).tolist()[0]
               
print(f"Output of predicted next {len(lst_output)} days/hours: {lstmdf}")
