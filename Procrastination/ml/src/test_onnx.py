# import numpy as np
# import onnxruntime as ort

# MODEL_PATH = "models/baseline_model.onnx"

# # [typing_speed, repetitive_key_ratio, mouse_velocity, idle_ratio, window_switch_frequency]

# def run_test():
#     print("Loading ONNX model...")

#     session = ort.InferenceSession(MODEL_PATH)

#     # Get input/output names
#     input_name = session.get_inputs()[0].name
#     output_name = session.get_outputs()[0].name

#     print("Input name:", input_name)
#     print("Output name:", output_name)

#     #  TEST CASES (simulate real user behavior)
#     test_inputs = [
#         # Focused
#         [3.0, 0.1, 100.0, 0.1],
        
#         # At Risk
#         [1.0, 0.2, 200.0, 0.3],
        
#         # Procrastinating
#         [0.3, 0.7, 400.0, 0.2],
        
#         # Idle
#         [0.0, 0.0, 0.0, 0.95]
#     ]

#     label_map = {
#         0: "Focused",
#         1: "At Risk",
#         2: "Procrastinating",
#         3: "Idle"
#     }

#     for i, features in enumerate(test_inputs):
#         print(f"\nTest Case {i+1}")
#         print("Input features:", features)

#         input_array = np.array([features], dtype=np.float32)

#         result = session.run([output_name], {input_name: input_array})

#         prediction = result[0][0]

#         print("Predicted label:", prediction, "-", label_map[prediction])


# if __name__ == "__main__":
#     run_test()



import onnxruntime as rt
sess = rt.InferenceSession("models/baseline_model.onnx")
print("INPUTS:")
for i in sess.get_inputs():
    print(f"  name={i.name}, shape={i.shape}, type={i.type}")
print("OUTPUTS:")
for o in sess.get_outputs():
    print(f"  name={o.name}, shape={o.shape}, type={o.type}")