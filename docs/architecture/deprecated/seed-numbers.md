# SpatialVortex Architecture

**Generating Spatial Vortex Machines from AI**
Generating Spatial Vortex Machines with commonly used Subjects, Keywords and Methods from Nomic Text Embedding and proven procedures discovered through AI API Keys returning a 9 step process for each machine.

Keeping the core 9 step process index 0 for each node.

Elaborating on each node with a positive and negative index for reinforcement learning (RL) to improve the inference predictatability and association with NLP, ML, and contextual AI reasoning.

This will create a morally superior process that is able to understand the moral implications of its actions and be able to correct itself, always aiming for the best possible outcome.

Physically this will create a standardized template that is repeated for each name it is invoked by.


**Mnmeonic Inference Engine**
The ability to feed seed numbers into a front end interface (Terminal to start) and have it generate an output derived from the subjects and seed numbers in reference to all the machines and matching nodes in the system with NLP, RL, ML, and AI reasoning.

**Seed Numbers**
888
872

**Subject Filter**
Specific
General Intelligence

**Model Tuning**
Nodes [Index]
Properties
Attributes
Parameters
State

Dynamics
Subject Comparison

**Tool Call**
Source
Lookup
Method

**Semantic Search**
NLP
ML
Knowledge Graphs
Words
Context
Can integrate Vector Search
Content Discovery

**Vector Search**
Chunks
RAG
Cache
Search Query
Keywords

**GraphQL**
API
Input
Output
Works with Semantic Search and Vector Search

# Super Station Inference System

Everytime a function is called in a specific API connected to the Super Station, it can recieve and return a JSON output with the inference results, leading to chain of thought and directly caused inferences by context, reasoning, RL and NNs.

Everytime a function is called, it requires variables for the function to execute

Invoking the Super Station Inference API requires a JSON input with the variables for the function to execute.
The JSON is then interpreted by Agents and call other Agents in the Super Station to preform the requirements and return abductive/lamda results in JSON.

The value in this is to interpret the JSON data and format it with rule based logic and return a JSON output with the final results, any additional data that may be missing for the system to be more comprehensive.

The client can then choose to use the additional results or update the attributes with the matching values.

**Example**
The client calls the API function with variables for a specific node.
The node's Properties, Hashtags, and Attributes are sent as a JSON table to the Super Station Inference API.

Queued Inference JSON
```JSON
{
    "ClassName": "Object",
    "Parent": "Space",
    "Children": {
        "TextLabel": {
            "Text": "John's Validity Check",
            "TextColor": "White",
            "TextSize": "16",
            "Font": "Arial",
            "DropShadow": true,
            "Bold": false,
            "Italic": false,
            "Underline": false,
            "Strikethrough": false,
            "Outline": false,
            "BackggroundTransparency": 1,
            "BackgroundColor": "Black",
            },
        },
    "Neighbors": { // Any Object within the Proximity will be added to the table
        "Enabled": "true", // Boolean determines if the table populates with nearby Objects
        "Discoverable": "true", // Boolean determines if other object's tables populate with this Object
        "Hashtag Filter": { // Allowlist that determines which objects based on Hashtags other Objects contain
            "Enabled": "true", // Boolean determines if the table populates with nearby Objects
            "Hashtags": [
                "Physical",
            ]
        },
        "Proximity": "100", // Meters of Proximity to populate the table
        "Objects": {
            "Object1": {
                "Name": "Object1",
                "Position": "(0,100,100)",
                "Distance": "100"
            },
            "Object2": {
                "Name": "Object2",
                "Position": "(100,100,0)",
                "Distance": "100"
            },
            "Object3": {
                "Name": "Object3",
                "Position": "(0,100,-100)",
                "Distance": "100"
            }
        }
    },
    "Properties": {
        "Name": "John's Validity Check",
        "Color": "Ultra White", // Color Names equal a preset list of RGB Values
        "Color3": "(255,255,255)", // RGB
        "Shape": "Box", // Tons of Three.js Options
        "Material": "Smooth", // Tons of Three.js Options with GLSL Shaders
        "Size": "(1,1,1)", // (Width,Height,Depth)
        "CFrame": {
            "Position": {"X": 0, "Y": 0, "Z": 0},
            "Rotation": {
                "Quaternion": {"X": 0, "Y": 0, "Z": 0, "W": 1}
            },
            "Matrix4x4": [
                [1, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 1]
            ],
            "LookVector": {"X": 0, "Y": 0, "Z": -1},
            "RightVector": {"X": 1, "Y": 0, "Z": 0},
            "UpVector": {"X": 0, "Y": 1, "Z": 0}
        }
    },
    "Physics": { // Canon.js
        "Type": "Static", // Dynamic or Static, can be physically simulated with forked Canon.js
        "Density": "1", // kg/m^3
        "Mass": "1", // Size * Density (kg)
        "Force of Gravity": "9.81m/s^2 * Mass", // Units: Newtons
        "Center of Gravity": "(0,0,0)", // Center offset by meters relative to the top and front of the object
        "Friction": "0.5", // Coefficient
        "Elasticity": "0.5", // Coefficient
        "Velocity": "(0,0,0)", // m/s
        "Angular Velocity": "(0,0,0)", // rad/s
        "Dynamic Position": { // A Physics based Object that can be added to Objects that keeps an object in the same position
            "Enabled": "true", // Boolean
            "Position": "(0,100,0)", // (X,Y,Z)
            "AtDestination": "false", // Boolean that determines if the object is at the destination
            "Proximity": "1", // Within one meter to determine AtDestination
            "MaxForce": "100", // Newtons
            "Force": "(0,0,0)", // (X,Y,Z) Current Force Vector
            "Torque": "(0,0,0)", // (X,Y,Z) Current Torque Vector
            "Angular Force": {// (X,Y,Z) Current Spin Vector, Applies to Center of Gravity
                "X": "0", // Spins the X axis +/- with Newtons
                "Y": "0", // Spins the Y axis +/- with Newtons
                "Z": "0", // Spins the Z axis +/- with Newtons
                "Dampening": "10", // Coefficient
                "Power": "1", // Coefficient
            },
        },
        "Dynamic Rotation": { // A Physics based Object that can be added to Objects
            "Rotation": "(0,0,0)", // (X,Y,Z) Rotation Force
            "MaxForce": "(0,0,0)", // (X,Y,Z) Maximum Force
            "Angular Force": "(0,0,0)", // (X,Y,Z) Current Angular Force
            "Torque": "(0,0,0)", // (X,Y,Z) Current Angular Torque
        },
    },
    "Hashtags": [
        "Abstract",
        "Static"
    ],
    "Attributes": {
        "Subject": "General Intelligence",
        "Student": "John Doe",
        "Teacher": "Dr. Smith",
        "Test": "The Essence of Creation",
        "Score": "Test Score: 90%",
        "Threshold": "70%",
        "Validity": ""
    },
    "Parameters": { // Parameters are Data Centric and all about CORS and control over CRUD operations
        "Ownership": {
            "Current Owner": "Simbuilder", // Who owns the object
            "Previous Owners": [
                "Bananaman9000"
            ]
        },
        "Possession": { // A table of who has had the object selected and when
            "Current Viewer": { // Who is currently viewing the object
                "Simbuilder": { // A Thinker Object
                   "Subscription": "Free Thinker", // Subscription Type
                   "Rank": "Viewer", // Rank Type: Viewer, Editor, Inventor
                   "Account Age": "365", // How many days the account is
                   "Reputation": { // Reputation Service, aquired by other thinkers enjoying, taking and commenting on your spaces, objects and think tanks. 
                        "Status": "100", // Spectrum Value 0-100, generated by actions done on the platform
                        "Capital": "$0", // Amount of capital generated
                        "Social Score": "40" // Perceieved value of the thinker from others
                   },
                }
            }, 
            "View History": {
                "Free Thinkers": {
                    "Simbuilder": {
                        "Date": "07/19/2025", // When the object was last viewed
                        "Time": "20:30:10", // When the object was last viewed
                        "Duration": "1.79", // How long the object was viewed in seconds
                    }
                },
                "Professionals": {
                    "Simbuilder": {
                        "Date": "07/19/2025", // When the object was last viewed
                        "Time": "20:30:10", // When the object was last viewed
                    }
                },
                "Enterprise": {
                    "Simbuilder": {
                        "Date": "07/19/2025", // When the object was last viewed
                        "Time": "20:30:10", // When the object was last viewed
                    }
                }
            },
            "Current Selection": "", // Who currently selected the object
            "Selected History": [
                "Simbuilder",
                "Builderman"
            ]
        },
        "Control": {
            "Domain": "",
            "Jurisdiction": "",
            "Filters": {
                "Logic": "",
                "Conditions": { // Conditions API allow Thinkers to CRUD Conditions that can be used to filter Objects
                    "Example": {
                        ""
                    }
                }
            },
            "Last Modified": "",
            "Created": {
                "Thinker": "Simbuilder",
                "Date": "07/19/2025",
                "Time": "8:41:47",
                
            },
            "Version": "1.0",
        }
        "Rank": {
            "Inventor": "Simbuilder", // Who has admin access to the object
                "Permissions": {
                    "Create": "true", // Boolean
                    "Read": "true", // Boolean
                    "Update": "true", // Boolean
                    "Delete": "true", // Boolean
                    "Execute": "true", // Boolean for running scripts
                    "Collect": "true", // Boolean for collecting the object to your inventory
                },
            "Editor": "Builderman", // Who has editor access to the object
                "Permissions": {
                    "Create": "true", // Boolean
                    "Read": "true", // Boolean
                    "Update": "true", // Boolean
                    "Delete": "true", // Boolean
                    "Execute": "true", // Boolean for running scripts
                    "Collect": "true", // Boolean for collecting the object to your inventory
                },
            "Viewer": "Simbuilder", // Who has viewer access to the object
                "Permissions": {
                    "Create": "true", // Boolean
                    "Read": "true", // Boolean
                    "Update": "true", // Boolean
                    "Delete": "true", // Boolean
                    "Execute": "true", // Boolean for running scripts
                    "Collect": "true", // Boolean for collecting the object to your inventory
                },
        },
       
    },
  
```

The Validity Check is a test to see if the System can reason with the JSON objects and return a valid JSON output with the final results, interpreting the data to decide if the Student passed the test or not.

If it works, it will return Validity: true while perserving the original JSON objects.

Parallel Processing (Rust API)
Nomic Text Embedding
Tool Call
Spatial Lookup API
Space
Location
Position
Neighboring

Bayes classification [https://en.wikipedia.org/wiki/Naive_Bayes_classifier]
Clustering from Super Station (K-Means)
k-Nearest Neighbors (kNN)
Q-Learning
Association Rule learning
Inferential Federated Learning Output (JSON)

Centric Selection Bias
SpatialVortex Rotation Mechanics
Viewport Constrainted Vector Ray Analysis

Chain of Thought
Parsing
Synonyms
Auto Completion
Mispelling Correction
Multi-Modal Input

Spatial Database
Vortex Inference System

Node Selection
Node Reference
Node Dimension +/-
Node Boolean
Node Weight
Node Bias
Node Index
Node Properties
Node Attributes
Node Parameters
Node State
Node Dynamics
Node Logic
Node Emotion
Node Permutations
Node Combinations
Node Proximity

Neural Networks
Selection Bias
Weights

Datalines
A = A
Start
Destination
Time

API
Request
Response
Performance

tRPC
gRPC
Python
SymPy
PyTorch
TensorFlow

Docker
Kubernetes

Terraform
EKS, GKE, AKS

The core index is 0 and is the center of the universe.

Node Reference
Node Dimension +/-
Node Index
Node Properties
Node Attributes

Nomic Local Model for Embedding Text powered by Ollama DeepSeek R1
Search
Query
Classification
Clustering
